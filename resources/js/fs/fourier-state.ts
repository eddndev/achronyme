
import { calculateCoefficients } from './fourier';
import * as math from 'mathjs';

// --- Interfaces de Tipos ---

interface SeriesCoefficients {
    a0: number;
    an: number[];
    bn: number[];
    period: number;
    domainStart: number;
}

// Datos que espera la librería del gráfico (definidos en app.ts)
interface FourierChartData {
    seriesCoeffs: SeriesCoefficients;
    functionDefinition: string;
    renderOriginal: boolean;
    renderSeries: boolean;
    terms_n: number;
}

// Estado completo del componente Alpine
interface FourierState {
    // Estado de la UI
    calculationMode: 'calculate' | 'coefficients';
    isLoading: boolean;
    errorMessage: string;

    // Modelo de datos
    functionDefinition: string;
    domainStart: string;
    domainEnd: string;
    coeff_a0_str: string;
    coeff_an_str: string;
    coeff_bn_str: string;
    renderOriginal: boolean;
    renderSeries: boolean;
    terms_n: number;
    seriesCoeffs: SeriesCoefficients;

    // Métodos
    init(): void;
    calculateAndRedraw(): Promise<void>;
    evaluateCoefficientExpressions(): void;
    prepareAndRedraw(recalculate?: boolean): void;
    getChartData(): FourierChartData;
    $watch: (property: string, callback: (value: any) => void) => void;
}

// --- Lógica del Componente ---

function fourierState(): FourierState {
    return {
        // Estado de la UI
        calculationMode: 'calculate',
        isLoading: false,
        errorMessage: '',

        // --- Modelo de datos ---
        functionDefinition: 't',
        domainStart: '-pi',
        domainEnd: 'pi',
        coeff_a0_str: '0',
        coeff_an_str: '(2*(-1)^(n+1))/n',
        coeff_bn_str: '0',
        renderOriginal: true,
        renderSeries: true,
        terms_n: 10,
        seriesCoeffs: {
            a0: 0,
            an: [],
            bn: [],
            period: 2 * Math.PI,
            domainStart: -Math.PI,
        },

        // --- Métodos ---
        init() {
            console.log('[Alpine] Initializing fourierState...');
            this.prepareAndRedraw();

            this.$watch('renderOriginal', () => this.prepareAndRedraw());
            this.$watch('renderSeries', () => this.prepareAndRedraw());
            this.$watch('terms_n', () => this.prepareAndRedraw(false));
        },

        async calculateAndRedraw() {
            this.isLoading = true;
            this.errorMessage = '';
            try {
                if (this.calculationMode === 'calculate') {
                    const result = calculateCoefficients(
                        this.functionDefinition,
                        this.domainStart,
                        this.domainEnd,
                        this.terms_n
                    );
                    if (result.success) {
                        this.seriesCoeffs = result.coeffs;
                    } else {
                        throw new Error(result.error || 'Ocurrió un error desconocido.');
                    }
                } else {
                    this.evaluateCoefficientExpressions();
                }
                this.prepareAndRedraw();
            } catch (error: any) {
                console.error('[Alpine] Calculation Error:', error);
                this.errorMessage = error.message;
            } finally {
                this.isLoading = false;
            }
        },

        evaluateCoefficientExpressions() {
            try {
                const a = math.evaluate(this.domainStart);
                const b = math.evaluate(this.domainEnd);
                this.seriesCoeffs.period = b - a;
                this.seriesCoeffs.domainStart = a;
                this.seriesCoeffs.a0 = math.evaluate(this.coeff_a0_str);

                const an_node = math.parse(this.coeff_an_str);
                const an_code = an_node.compile();
                const bn_node = math.parse(this.coeff_bn_str);
                const bn_code = bn_node.compile();

                this.seriesCoeffs.an = [0];
                this.seriesCoeffs.bn = [0];

                for (let n = 1; n <= this.terms_n; n++) {
                    this.seriesCoeffs.an[n] = an_code.evaluate({ n, pi: Math.PI });
                    this.seriesCoeffs.bn[n] = bn_code.evaluate({ n, pi: Math.PI });
                }
            } catch (error: any) {
                throw new Error(`Error al evaluar expresiones: ${error.message}`);
            }
        },

        prepareAndRedraw(recalculate = true) {
            if (this.calculationMode === 'coefficients' && recalculate) {
                try {
                    this.evaluateCoefficientExpressions();
                } catch (error: any) {
                    this.errorMessage = error.message;
                    return;
                }
            }
            window.FourierSeriesChart.redraw(this.getChartData());
        },

        getChartData(): FourierChartData {
            return {
                seriesCoeffs: this.seriesCoeffs,
                functionDefinition: this.functionDefinition,
                renderOriginal: this.renderOriginal,
                renderSeries: this.renderSeries,
                terms_n: this.terms_n,
            };
        },
    } as FourierState;
}

// Extender la interfaz global de Window para incluir nuestra función
declare global {
    interface Window {
        fourierState: () => FourierState;
    }
}

window.fourierState = fourierState;
