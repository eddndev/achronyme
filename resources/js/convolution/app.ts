import './convolution-state';
import { Chart, registerables, ChartConfiguration } from 'chart.js';

Chart.register(...registerables);

// --- Functions Chart (f and g) ---
window.FunctionsChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('functionsChart') as HTMLCanvasElement;
        const container = document.getElementById('functionsContainer');

        if (!ctx || !container) {
            console.error('Functions chart canvas or container not found!');
            return;
        }

        if (this.chart) {
            this.chart.destroy();
        }

        ctx.style.width = '100%';
        ctx.style.height = '100%';
        ctx.style.maxWidth = '100%';

        const chartConfig: ChartConfiguration = {
            type: 'line',
            data: { labels: [], datasets: [] },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                scales: {
                    x: {
                        title: { display: true, text: 'τ' },
                        ticks: {
                            maxTicksLimit: 15,
                            callback: function(value, index, ticks) {
                                const num = parseFloat(this.getLabelForValue(value as number));
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(2);
                                }
                                return num.toFixed(2);
                            }
                        }
                    },
                    y: {
                        title: { display: true, text: 'Amplitud' },
                        ticks: {
                            callback: function(value) {
                                const num = value as number;
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(2);
                                }
                                return num.toFixed(2);
                            }
                        }
                    }
                },
                plugins: {
                    legend: { position: 'top' },
                    tooltip: { mode: 'index', intersect: false }
                },
                elements: { point: { radius: 0 } }
            }
        };

        this.chart = new Chart(ctx, chartConfig);
    },

    redraw(data: { tau: number[], f: number[], g: number[], product?: number[] }) {
        if (!this.chart) return;

        this.chart.data.labels = data.tau.map(t => t.toFixed(2));

        const datasets: any[] = [];

        // f(τ) dataset
        if (data.f) {
            datasets.push({
                label: 'f(τ)',
                data: data.f,
                borderColor: 'rgb(59, 130, 246)',
                borderWidth: 2,
                tension: 0.1
            });
        }

        // g(t-τ) dataset
        if (data.g) {
            datasets.push({
                label: 'g(t-τ)',
                data: data.g,
                borderColor: 'rgb(34, 197, 94)',
                borderWidth: 2,
                tension: 0.1
            });
        }

        // Product f·g dataset (area fill)
        if (data.product) {
            datasets.push({
                label: 'f(τ)·g(t-τ)',
                data: data.product,
                borderColor: 'rgb(168, 85, 247)',
                backgroundColor: 'rgba(168, 85, 247, 0.2)',
                borderWidth: 1,
                fill: true,
                tension: 0.1
            });
        }

        this.chart.data.datasets = datasets;
        this.chart.update('none');
    },

    clear() {
        if (!this.chart) return;
        this.chart.data.labels = [];
        this.chart.data.datasets = [];
        this.chart.update('none');
    }
};

// --- Convolution Result Chart ---
window.ResultChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('resultChart') as HTMLCanvasElement;
        const container = document.getElementById('resultContainer');

        if (!ctx || !container) {
            console.error('Result chart canvas or container not found!');
            return;
        }

        if (this.chart) {
            this.chart.destroy();
        }

        ctx.style.width = '100%';
        ctx.style.height = '100%';
        ctx.style.maxWidth = '100%';

        const chartConfig: ChartConfiguration = {
            type: 'line',
            data: { datasets: [] },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                scales: {
                    x: {
                        type: 'linear',
                        title: { display: true, text: 't' },
                        ticks: {
                            maxTicksLimit: 15,
                            callback: function(value) {
                                const num = value as number;
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(2);
                                }
                                return num.toFixed(2);
                            }
                        }
                    },
                    y: {
                        title: { display: true, text: '(f*g)(t)' },
                        ticks: {
                            callback: function(value) {
                                const num = value as number;
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(2);
                                }
                                return num.toFixed(2);
                            }
                        }
                    }
                },
                plugins: {
                    legend: { position: 'top' },
                    tooltip: { mode: 'index', intersect: false }
                },
                elements: { point: { radius: 0 } }
            }
        };

        this.chart = new Chart(ctx, chartConfig);
    },

    redraw(data: { t: number[], result: number[], currentPoint?: { t: number, value: number } }) {
        if (!this.chart) return;

        const datasets: any[] = [];

        // Main convolution result - use x,y pairs for proper positioning
        datasets.push({
            label: '(f*g)(t)',
            data: data.t.map((t, i) => ({ x: t, y: data.result[i] })),
            borderColor: 'rgb(239, 68, 68)',
            backgroundColor: 'rgba(239, 68, 68, 0.1)',
            borderWidth: 2,
            fill: true,
            tension: 0.1,
            showLine: true
        });

        // Current point marker (always show using scatter)
        if (data.currentPoint) {
            // Create a scatter dataset with a single point
            const tValue = typeof data.currentPoint.t === 'number' ? data.currentPoint.t : parseFloat(data.currentPoint.t);
            datasets.push({
                type: 'scatter',
                label: 'Punto actual',
                data: [{
                    x: tValue,
                    y: data.currentPoint.value
                }],
                borderColor: 'rgb(251, 191, 36)',
                backgroundColor: 'rgb(251, 191, 36)',
                pointRadius: 8,
                pointHoverRadius: 10,
                pointStyle: 'circle'
            });
        }

        this.chart.data.datasets = datasets;
        this.chart.update('none');
    },

    clear() {
        if (!this.chart) return;
        this.chart.data.labels = [];
        this.chart.data.datasets = [];
        this.chart.update('none');
    }
};

// --- Global Type Declarations ---
declare global {
    interface Window {
        FunctionsChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: { tau: number[], f: number[], g: number[], product?: number[] }) => void;
            clear: () => void;
        };
        ResultChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: { t: number[], result: number[], currentPoint?: { t: number, value: number } }) => void;
            clear: () => void;
        };
    }
}