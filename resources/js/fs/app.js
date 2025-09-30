import { Chart, registerables } from 'chart.js';
Chart.register(...registerables);

function initFourierChart() {
    const ctx = document.getElementById('fourierChart');
    if (!ctx) {
        return; // Exit if the chart canvas is not on the page
    }

    const chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            animation: {
                duration: 400 // Smoother animation
            },
            scales: {
                x: {
                    title: { display: true, text: 't' },
                    ticks: { maxTicksLimit: 15 }
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
            // Create a function with a restricted scope to Math functions.
            const func = new Function('t', `
                const { sin, cos, tan, exp, pow, PI } = Math;
                const pi = PI;
                return ${funcStr.replace(/\^/g, '**')};
            `);
            const result = func(t);
            return isFinite(result) ? result : 0;
        } catch (e) {
            console.error("Error evaluating function:", e);
            return 0; // Return 0 on any parsing/evaluation error
        }
    };

    Livewire.on('fourier-series-updated', ({ seriesCoeffs, domainStart, domainEnd, functionDefinition, renderOriginal, renderSeries }) => {
        const STEPS = 400;
        const stepSize = (domainEnd - domainStart) / STEPS;
        const labels = [];
        const seriesData = [];
        const originalData = [];

        const T = domainEnd - domainStart;
        if (T <= 0) return; // Avoid division by zero or infinite loops

        for (let i = 0; i <= STEPS; i++) {
            const t = domainStart + i * stepSize;
            labels.push(t.toFixed(2));

            // 1. Reconstruct the Fourier Series signal if requested
            if (renderSeries) {
                // This is the key logic from the refactoring plan
                let y = seriesCoeffs.a0 / 2;
                const N = seriesCoeffs.an ? Object.keys(seriesCoeffs.an).length : 0;

                for (let n = 1; n <= N; n++) {
                    const an = seriesCoeffs.an[n] || 0;
                    const bn = seriesCoeffs.bn[n] || 0;
                    y += an * Math.cos(2 * Math.PI * n * t / T);
                    y += bn * Math.sin(2 * Math.PI * n * t / T);
                }
                seriesData.push(y);
            }

            // 2. Evaluate the original function if requested
            if (renderOriginal) {
                originalData.push(evaluateFunction(functionDefinition, t));
            }
        }

        // 3. Update chart datasets
        const datasets = [];
        if (renderSeries) {
            datasets.push({
                label: 'Serie de Fourier',
                data: seriesData,
                borderColor: 'rgb(234, 179, 8)',
                borderWidth: 2,
                tension: 0.1
            });
        }
        if (renderOriginal) {
            datasets.push({
                label: 'Función Original',
                data: originalData,
                borderColor: 'rgb(59, 130, 246)',
                borderWidth: 2,
                tension: 0.1,
                borderDash: [5, 5] // Dashed line for original function
            });
        }

        chart.data.labels = labels;
        chart.data.datasets = datasets;
        chart.update();
    });
}

// Ensure Livewire is initialized before we set up our listeners
document.addEventListener('livewire:init', () => {
    initFourierChart();
});