import { Chart, registerables } from 'chart.js';
Chart.register(...registerables);

function initFourierChart() {
    const ctx = document.getElementById('fourierChart');
    if (!ctx) {
        return; // Exit if the chart canvas is not on the page
    }

    // Initialize a global chart instance
    const chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: false, // Disable animations for smoother real-time updates
            scales: {
                x: {
                    title: { display: true, text: 't' },
                    ticks: { maxTicksLimit: 20 }
                },
                y: {
                    title: { display: true, text: 'f(t)' }
                }
            },
            plugins: {
                legend: { position: 'top' },
                tooltip: { mode: 'index', intersect: false }
            },
            elements: {
                point: {
                    radius: 0 // Hide points for a smooth line
                }
            }
        }
    });

    // A simple, safer function evaluator for the frontend rendering.
    const evaluateFunction = (funcStr, t) => {
        try {
            const func = new Function('t', `
                const { sin, cos, tan, exp, pow, PI } = Math;
                const pi = PI;
                return ${funcStr.replace(/\^/g, '**')};
            `);
            const result = func(t);
            return isFinite(result) ? result : 0;
        } catch (e) {
            console.error("Error evaluating function:", e);
            return 0;
        }
    };

    /**
     * Redraws the entire chart based on the latest data from the Livewire component.
     * @param {object} data The component's snapshot data.
     */
    const redrawFourierSeries = (data) => {
        if (!data || !data.seriesCoeffs) return;

        const { seriesCoeffs, functionDefinition, renderOriginal, renderSeries, terms_n } = data;
        const { a0, an, bn, period, domainStart } = seriesCoeffs;

        // If there's no period, clear the chart and exit.
        if (!period || period <= 0) {
            chart.data.labels = [];
            chart.data.datasets = [];
            chart.update();
            return;
        }

        const STEPS = 500; // Increased steps for a smoother curve over two periods
        const plotDomainStart = domainStart;
        const plotDomainEnd = domainStart + 2 * period; // Plot over two full periods
        const stepSize = (plotDomainEnd - plotDomainStart) / STEPS;

        const labels = [];
        const seriesData = [];
        const originalData = [];

        for (let i = 0; i <= STEPS; i++) {
            const t = plotDomainStart + i * stepSize;
            labels.push(t.toFixed(2));

            // 1. Reconstruct the Fourier Series signal if requested
            if (renderSeries) {
                let y = a0 / 2;
                const limit = terms_n; // Use the slider's current value
                for (let n = 1; n <= limit; n++) {
                    if (an[n] !== undefined && bn[n] !== undefined) {
                        y += an[n] * Math.cos(2 * Math.PI * n * t / period);
                        y += bn[n] * Math.sin(2 * Math.PI * n * t / period);
                    }
                }
                seriesData.push(y);
            }

            // 2. Evaluate the original function (made periodic) if requested
            if (renderOriginal) {
                // Map `t` from the extended domain back to the base domain [domainStart, domainStart + period)
                let t_mod = t;
                while (t_mod >= domainStart + period) { t_mod -= period; }
                while (t_mod < domainStart) { t_mod += period; }
                originalData.push(evaluateFunction(functionDefinition, t_mod));
            }
        }

        // 3. Update chart datasets
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

        chart.data.labels = labels;
        chart.data.datasets = datasets;
        chart.update('none'); // 'none' prevents animation, making slider updates smooth
    };

    // Use Livewire hooks for robust component lifecycle management
    Livewire.hook('component:init', ({ component, cleanup }) => {
        // This hook is for a specific component, check its name
        if (component.name !== 'fourier-series-tool') return;

        // Initial draw when the component is loaded
        redrawFourierSeries(component.snapshot.data);

        // Listen for updates (e.g., when the slider is moved)
        const handleUpdate = () => {
            redrawFourierSeries(component.snapshot.data);
        };
        component.on('updated', handleUpdate);

        // Clean up the listener when the component is destroyed
        cleanup(() => {
            // In a real SPA, you'd remove the listener here.
        });
    });
}

// Ensure Livewire is fully initialized before we set up our listeners
document.addEventListener('livewire:init', () => {
    initFourierChart();
});