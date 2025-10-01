import './fourier-state';
import { Chart, registerables, ChartConfiguration } from 'chart.js';
import * as math from 'mathjs'; // Import math.js
Chart.register(...registerables);

// Define an interface for the data structure expected by the chart functions
interface FourierSeriesChartData {
    seriesCoeffs: {
        a0: number;
        an: number[];
        bn: number[];
        period: number;
        domainStart: number;
    };
    functionDefinition: string;
    renderOriginal: boolean;
    renderSeries: boolean;
    terms_n: number;
}

// Objeto global para manejar la lógica del gráfico
window.FourierSeriesChart = {
    chart: null as Chart | null, // Mantendrá la instancia de Chart.js

    /**
     * Inicializa el gráfico de la Serie de Fourier.
     * @param {FourierSeriesChartData} initialData - Los datos iniciales para dibujar el gráfico.
     */
    init(initialData: FourierSeriesChartData) {
        console.log('[FS_APP] Initializing chart with data:', initialData);
        const ctx = document.getElementById('fourierChart') as HTMLCanvasElement;
        if (!ctx) {
            console.error('[FS_APP] Chart canvas #fourierChart not found! Aborting.');
            return;
        }

        // Destruye el gráfico anterior si existe, para evitar conflictos
        if (this.chart) {
            this.chart.destroy();
        }

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

        // Dibuja el gráfico con los datos iniciales
        this.redraw(initialData);
    },

    /**
     * Redibuja el gráfico completo basado en los datos más recientes.
     * @param {FourierSeriesChartData} data - El objeto de datos completo del componente.
     */
    redraw(data: FourierSeriesChartData) {
        console.log('[FS_APP] Redrawing chart with data:', data);
        if (data && data.seriesCoeffs) {
            console.log('[FS_APP] Received seriesCoeffs:', JSON.parse(JSON.stringify(data.seriesCoeffs)));
        }

        if (!this.chart || !data || !data.seriesCoeffs) {
            console.warn('[FS_APP] Redraw failed: Chart instance or data is missing.');
            return;
        }

        const { seriesCoeffs, functionDefinition, renderOriginal, renderSeries, terms_n } = data;
        const { a0, an, bn, period, domainStart } = seriesCoeffs;

        if (!period || period <= 0) {
            console.warn('[FS_APP] Period is zero or invalid. Clearing chart.');
            this.chart.data.labels = [];
            this.chart.data.datasets = [];
            this.chart.update();
            return;
        }

        const STEPS = 500;
        const plotDomainStart = domainStart;
        const plotDomainEnd = domainStart + 2 * period;
        const stepSize = (plotDomainEnd - plotDomainStart) / STEPS;

        const labels: string[] = [];
        const seriesData: number[] = [];
        const originalData: number[] = [];

        // Compile the user function once for efficiency
        let compiledUserFunc: math.EvalFunction | null = null;
        try {
            const node = math.parse(functionDefinition);
            compiledUserFunc = node.compile();
        } catch (e) {
            console.error("[FS_APP] Error compiling function definition:", e);
            // If compilation fails, we can't plot the original function
            // and should probably clear originalData or handle this gracefully.
        }


        for (let i = 0; i <= STEPS; i++) {
            const t = plotDomainStart + i * stepSize;
            labels.push(t.toFixed(2));

            if (renderSeries) {
                let y = a0 / 2;
                const limit = terms_n;
                for (let n = 1; n <= limit; n++) {
                    if (an[n] !== undefined && bn[n] !== undefined) {
                        y += an[n] * Math.cos(2 * Math.PI * n * t / period);
                        y += bn[n] * Math.sin(2 * Math.PI * n * t / period);
                    }
                }
                seriesData.push(y);
            }

            if (renderOriginal && compiledUserFunc) {
                let t_mod = t;
                while (t_mod >= domainStart + period) { t_mod -= period; }
                while (t_mod < domainStart) { t_mod += period; }
                originalData.push(this.evaluateCompiledFunction(compiledUserFunc, t_mod));
            }
        }

        const datasets = [];
        if (renderSeries) {
            datasets.push({
                label: `Serie de Fourier (N=${terms_n})`,
                data: seriesData,
                borderColor: 'rgb(234, 179, 8)',
                borderWidth: 2,
                tension: 0.1
            });
        }
        if (renderOriginal) {
            datasets.push({
                label: 'Función Original (Periódica)',
                data: originalData,
                borderColor: 'rgb(59, 130, 246)',
                borderWidth: 2,
                tension: 0.1,
                borderDash: [5, 5]
            });
        }

        this.chart.data.labels = labels;
        this.chart.data.datasets = datasets;
        this.chart.update('none');
    },

    /**
     * Evalúa una función compilada de math.js de forma segura.
     */
    evaluateCompiledFunction(compiledFunc: math.EvalFunction, t: number): number {
        try {
            const result = compiledFunc.evaluate({ t, pi: Math.PI });
            return typeof result === 'number' && isFinite(result) ? result : 0;
        } catch (e) {
            console.error("[FS_APP] Error evaluating compiled function:", e);
            return 0;
        }
    }
};

// Extend the Window interface to include FourierSeriesChart
declare global {
    interface Window {
        FourierSeriesChart: {
            chart: Chart | null;
            init: (initialData: FourierSeriesChartData) => void;
            redraw: (data: FourierSeriesChartData) => void;
            evaluateCompiledFunction: (compiledFunc: math.EvalFunction, t: number) => number;
        };
    }
}
