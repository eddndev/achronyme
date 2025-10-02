import './fourier-transform-state';
import { Chart, registerables, ChartConfiguration } from 'chart.js';
import * as math from 'mathjs';

Chart.register(...registerables);

// --- Interfaces ---
interface FourierTransformData {
    timeDomain: {
        t: number[];
        values: number[];
    };
    frequencyDomain: {
        omega: number[];
        magnitude: number[];
        phase: number[];
    };
}

// --- Time Domain Chart ---
window.TimeDomainChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('timeDomainChart') as HTMLCanvasElement;
        const container = document.getElementById('timeDomainContainer');

        if (!ctx || !container) {
            console.error('Time domain chart canvas or container not found!');
            return;
        }

        if (this.chart) {
            this.chart.destroy();
        }

        // Set canvas dimensions
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
                        title: { display: true, text: 't (tiempo)' },
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
                        title: { display: true, text: 'f(t)' },
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

    redraw(data: { t: number[]; values: number[] }) {
        if (!this.chart) return;

        this.chart.data.labels = data.t.map(t => t.toFixed(2));
        this.chart.data.datasets = [{
            label: 'f(t)',
            data: data.values,
            borderColor: 'rgb(59, 130, 246)',
            borderWidth: 2,
            tension: 0.1
        }];

        this.chart.update('none');
    },

    clear() {
        if (!this.chart) return;
        this.chart.data.labels = [];
        this.chart.data.datasets = [];
        this.chart.update('none');
    }
};

// --- Magnitude Spectrum Chart ---
window.MagnitudeChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('magnitudeChart') as HTMLCanvasElement;
        const container = document.getElementById('magnitudeContainer');

        if (!ctx || !container) {
            console.error('Magnitude chart canvas or container not found!');
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
                        title: { display: true, text: 'ω (rad/s)' },
                        ticks: {
                            maxTicksLimit: 10,
                            callback: function(value, index, ticks) {
                                const num = parseFloat(this.getLabelForValue(value as number));
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(1);
                                }
                                return num.toFixed(1);
                            }
                        }
                    },
                    y: {
                        title: { display: true, text: '|F(ω)|' },
                        ticks: {
                            callback: function(value) {
                                const num = value as number;
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(1);
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

    redraw(data: { omega: number[]; magnitude: number[] }) {
        if (!this.chart) return;

        this.chart.data.labels = data.omega.map(w => w.toFixed(2));
        this.chart.data.datasets = [{
            label: '|F(ω)|',
            data: data.magnitude,
            borderColor: 'rgb(34, 197, 94)',
            backgroundColor: 'rgba(34, 197, 94, 0.1)',
            borderWidth: 2,
            fill: true,
            tension: 0.1
        }];

        this.chart.update('none');
    },

    clear() {
        if (!this.chart) return;
        this.chart.data.labels = [];
        this.chart.data.datasets = [];
        this.chart.update('none');
    }
};

// --- Phase Spectrum Chart ---
window.PhaseChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('phaseChart') as HTMLCanvasElement;
        const container = document.getElementById('phaseContainer');

        if (!ctx || !container) {
            console.error('Phase chart canvas or container not found!');
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
                        title: { display: true, text: 'ω (rad/s)' },
                        ticks: {
                            maxTicksLimit: 10,
                            callback: function(value, index, ticks) {
                                const num = parseFloat(this.getLabelForValue(value as number));
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(1);
                                }
                                return num.toFixed(1);
                            }
                        }
                    },
                    y: {
                        title: { display: true, text: '∠F(ω) (rad)' },
                        ticks: {
                            callback: function(value) {
                                const num = value as number;
                                if (Math.abs(num) < 1e-10) return '0';
                                if (Math.abs(num) < 0.01 || Math.abs(num) > 1000) {
                                    return num.toExponential(1);
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

    redraw(data: { omega: number[]; phase: number[] }) {
        if (!this.chart) return;

        this.chart.data.labels = data.omega.map(w => w.toFixed(2));
        this.chart.data.datasets = [{
            label: '∠F(ω)',
            data: data.phase,
            borderColor: 'rgb(168, 85, 247)',
            backgroundColor: 'rgba(168, 85, 247, 0.1)',
            borderWidth: 2,
            fill: true,
            tension: 0.1
        }];

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
        TimeDomainChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: { t: number[]; values: number[] }) => void;
            clear: () => void;
        };
        MagnitudeChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: { omega: number[]; magnitude: number[] }) => void;
            clear: () => void;
        };
        PhaseChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: { omega: number[]; phase: number[] }) => void;
            clear: () => void;
        };
    }
}