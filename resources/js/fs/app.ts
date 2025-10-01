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
        if (!this.chart || !data || !data.functions) {
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
        const shouldRenderOriginal = renderOriginal.includes('original') && functions.length > 0;

        const datasets = [];
        const allDataPoints: number[] = [];
        const STEPS_PER_PERIOD = 500;

        // --- Domain Calculation ---
        let plotDomainStart: number;
        let plotDomainEnd: number;

        // If we are showing the original function, its domain is the authority.
        // Otherwise, if we are only showing the series, its domain is the authority.
        if (shouldRenderOriginal) {
            plotDomainStart = functions.length > 0 ? math.evaluate(functions[0].domainStart) : 0;
            plotDomainEnd = functions.length > 0 ? math.evaluate(functions[functions.length - 1].domainEnd) : 1;
        } else if (shouldRenderSeries) {
            plotDomainStart = piecewiseCoeffs!.domainStart;
            plotDomainEnd = piecewiseCoeffs!.domainStart + piecewiseCoeffs!.period;
        } else {
            // Default fallback domain if nothing is being rendered
            plotDomainStart = -Math.PI;
            plotDomainEnd = Math.PI;
        }

        const totalPeriod = plotDomainEnd - plotDomainStart;
        const stepSize = totalPeriod > 0 ? totalPeriod / STEPS_PER_PERIOD : 0.1;

        const labels: string[] = [];
        for (let t = plotDomainStart; t <= plotDomainEnd; t += stepSize) {
            labels.push(t.toFixed(2));
        }

        // --- Original Function(s) Plotting ---
        if (shouldRenderOriginal) {
            // Compile all functions first and store them with their domains
            const compiledFunctions = functions.map(func => {
                try {
                    return {
                        compiled: math.parse(func.definition).compile(),
                        start: math.evaluate(func.domainStart),
                        end: math.evaluate(func.domainEnd)
                    };
                } catch (e) {
                    return null;
                }
            }).filter(f => f !== null) as { compiled: math.EvalFunction; start: number; end: number }[];

            // Create a single data array for the entire piecewise function
            const originalData: (number | null)[] = [];

            // Iterate through all time steps (labels)
            for (const tStr of labels) {
                const t = parseFloat(tStr);
                let value: number | null = null;

                // Find the correct function piece for the current time t
                const activeFunc = compiledFunctions.find(f => t >= f.start - 1e-9 && t <= f.end + 1e-9);

                if (activeFunc) {
                    const val = this.evaluateCompiledFunction(activeFunc.compiled, t);
                    if (isFinite(val)) {
                        value = val;
                        allDataPoints.push(val);
                    }
                }
                originalData.push(value);
            }

            // Push a single dataset for the original function
            datasets.push({
                label: `f(t)`,
                data: originalData,
                borderColor: 'rgb(59, 130, 246)',
                borderWidth: 2,
                borderDash: [5, 5],
                tension: 0.1
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