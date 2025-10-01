import './fourier-state';
import { Chart, registerables, ChartConfiguration } from 'chart.js';
import * as math from 'mathjs';

Chart.register(...registerables);

// --- Interfaces --- 
interface SeriesCoefficients {
    a0: number;
    an: number[];
    bn: number[];
    period: number;
    domainStart: number;
}

interface FunctionInput {
    id: number;
    definition: string;
    domainStart: string;
    domainEnd: string;
}

interface FourierChartData {
    functions: FunctionInput[];
    renderOriginal: string[];
    renderSeries: string[];
    terms_n: number;
    piecewiseCoeffs: SeriesCoefficients | null; // The single source of truth for the series
}

// --- Charting Object ---
window.FourierSeriesChart = {
    chart: null as Chart | null,
    currentMinY: null as number | null,
    currentMaxY: null as number | null,

    resetScales() {
        this.currentMinY = null;
        this.currentMaxY = null;
    },

    init(initialData: FourierChartData) {
        const ctx = document.getElementById('fourierChart') as HTMLCanvasElement;
        if (!ctx) {
            console.error('Chart canvas #fourierChart not found!');
            return;
        }

        if (this.chart) {
            this.chart.destroy();
        }

        this.resetScales();

        const chartConfig: ChartConfiguration = {
            type: 'line',
            data: { labels: [], datasets: [] },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                scales: {
                    x: { title: { display: true, text: 't' }, ticks: { maxTicksLimit: 20 } },
                    y: { title: { display: true, text: 'f(t)' } }
                },
                plugins: {
                    legend: { position: 'top' },
                    tooltip: { mode: 'index', intersect: false }
                },
                elements: { point: { radius: 0 } }
            }
        };

        this.chart = new Chart(ctx, chartConfig);
        this.redraw(initialData);
    },

    redraw(data: FourierChartData) {
        if (!this.chart || !data || !data.functions || data.functions.length === 0) {
            console.warn('Redraw failed: Chart instance or data is missing.');
            if(this.chart) {
                this.chart.data.labels = [];
                this.chart.data.datasets = [];
                this.chart.update('none');
            }
            return;
        }

        const { functions, renderOriginal, renderSeries, terms_n, piecewiseCoeffs } = data;
        const shouldRenderSeries = renderSeries.includes('series') && !!piecewiseCoeffs;
        const shouldRenderOriginal = renderOriginal.includes('original');

        const datasets = [];
        const allDataPoints: number[] = [];
        const STEPS_PER_PERIOD = 500;
        const colors = ['rgb(59, 130, 246)', 'rgb(234, 179, 8)', 'rgb(16, 185, 129)', 'rgb(239, 68, 68)', 'rgb(139, 92, 246)'];

        // Determine the full plotting range from the functions array
        const plotDomainStart = math.evaluate(functions[0].domainStart);
        const plotDomainEnd = math.evaluate(functions[functions.length - 1].domainEnd);
        const totalPeriod = plotDomainEnd - plotDomainStart;
        const stepSize = totalPeriod > 0 ? totalPeriod / STEPS_PER_PERIOD : 0.1;

        const labels: string[] = [];
        for (let t = plotDomainStart; t <= plotDomainEnd; t += stepSize) {
            labels.push(t.toFixed(2));
        }

        // --- Original Function(s) Plotting ---
        if (shouldRenderOriginal) {
            functions.forEach((func, index) => {
                const domainStart = math.evaluate(func.domainStart);
                const domainEnd = math.evaluate(func.domainEnd);
                const color = colors[index % colors.length];

                const originalData: (number | null)[] = [];
                let compiledUserFunc: math.EvalFunction | null = null;
                try { compiledUserFunc = math.parse(func.definition).compile(); } catch (e) { /* ignore */ } 

                if (compiledUserFunc) {
                    for (const tStr of labels) {
                        const t = parseFloat(tStr);
                        // Use a small tolerance for floating point comparisons
                        if (t >= domainStart - 1e-9 && t <= domainEnd + 1e-9) {
                            const val = this.evaluateCompiledFunction(compiledUserFunc, t);
                            originalData.push(val);
                            if (isFinite(val)) allDataPoints.push(val);
                        } else {
                            originalData.push(null); // Gap in data
                        }
                    }
                    datasets.push({
                        label: `f(t) ${index + 1}`,
                        data: originalData,
                        borderColor: color,
                        borderWidth: 2,
                        borderDash: [5, 5],
                        tension: 0.1
                    });
                }
            });
        }

        // --- Single Fourier Series Plotting ---
        if (shouldRenderSeries) {
            const { a0, an, bn, period } = piecewiseCoeffs!;
            const seriesData: number[] = [];
            for (const tStr of labels) {
                const t = parseFloat(tStr);
                let y = a0 / 2;
                for (let n = 1; n <= terms_n; n++) {
                    if (an[n] !== undefined && bn[n] !== undefined) {
                        // The formula uses the overall period of the piecewise function
                        y += an[n] * Math.cos(2 * Math.PI * n * t / period);
                        y += bn[n] * Math.sin(2 * Math.PI * n * t / period);
                    }
                }
                seriesData.push(y);
                if (isFinite(y)) allDataPoints.push(y);
            }
            datasets.push({
                label: `Serie de Fourier (N=${terms_n})`,
                data: seriesData,
                borderColor: 'rgb(220, 38, 38)', // A distinct color for the series
                borderWidth: 2,
                tension: 0.1
            });
        }

        // --- Y-Axis Stabilization ---
        if (allDataPoints.length > 0) {
            let minVal = Math.min(...allDataPoints);
            let maxVal = Math.max(...allDataPoints);

            if (this.currentMinY === null || minVal < this.currentMinY) this.currentMinY = minVal;
            if (this.currentMaxY === null || maxVal > this.currentMaxY) this.currentMaxY = maxVal;

            let finalMin = this.currentMinY;
            let finalMax = this.currentMaxY;

            if (finalMin === finalMax) { finalMin -= 1; finalMax += 1; }

            const range = finalMax - finalMin;
            const padding = range * 0.1;

            this.chart.options.scales.y.min = finalMin - padding;
            this.chart.options.scales.y.max = finalMax + padding;
        } else {
            this.resetScales();
            delete this.chart.options.scales.y.min;
            delete this.chart.options.scales.y.max;
        }

        this.chart.data.labels = labels;
        this.chart.data.datasets = datasets;
        this.chart.update('none');
    },

    evaluateCompiledFunction(compiledFunc: math.EvalFunction, t: number): number {
        try {
            const result = compiledFunc.evaluate({ t, pi: Math.PI });
            return typeof result === 'number' && isFinite(result) ? result : NaN;
        } catch (e) {
            console.error("Error evaluating compiled function:", e);
            return NaN;
        }
    }
};

declare global {
    interface Window {
        FourierSeriesChart: {
            chart: Chart | null;
            currentMinY: number | null;
            currentMaxY: number | null;
            init: (initialData: FourierChartData) => void;
            redraw: (data: FourierChartData) => void;
            resetScales: () => void;
            evaluateCompiledFunction: (compiledFunc: math.EvalFunction, t: number) => number;
        };
    }
}