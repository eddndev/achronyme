import { calculateCoefficients } from './fourier';
import * as math from 'mathjs';
import { validateConstant, validateFunction } from './validation';

// --- Interfaces de Tipos ---

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
    definitionError: string | null;
    domainStartError: string | null;
    domainEndError: string | null;
}

// Datos que espera la librería del gráfico (definidos en app.ts)
interface FourierChartData {
    functions: FunctionInput[];
    renderOriginal: string[];
    renderSeries: string[];
    terms_n: number;
    piecewiseCoeffs: SeriesCoefficients | null;
}

// Estado completo del componente Alpine
interface FourierState {
    // Estado de la UI
    calculationMode: 'calculate' | 'coefficients';
    isLoading: boolean;
    errorMessage: string;

    // Modelo de datos
    functions: FunctionInput[];
    nextId: number;
    piecewiseCoeffs: SeriesCoefficients | null;
    coeff_a0_str: string;
    coeff_an_str: string;
    coeff_bn_str: string;
    renderOriginal: string[];
    renderSeries: string[];
    terms_n: number;

    // Métodos
    init(): void;
    addFunction(): void;
    removeFunction(id: number): void;
    validate(): boolean;
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
        nextId: 2,
        functions: [
            {
                id: 1,
                definition: 't',
                domainStart: '-pi',
                domainEnd: 'pi',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            }
        ],
        piecewiseCoeffs: null,
        coeff_a0_str: '0',
        coeff_an_str: '(2*(-1)^(n+1))/n',
        coeff_bn_str: '0',
        renderOriginal: ['original'],
        renderSeries: ['series'],
        terms_n: 10,

        // --- Métodos ---
        init() {
            console.log('[Alpine] Initializing fourierState...');

            // First, initialize the chart with empty data. This ensures the chart object
            // exists before we try to draw the actual data on it.
            window.FourierSeriesChart.init({
                functions: [],
                renderOriginal: [],
                renderSeries: [],
                terms_n: this.terms_n,
                piecewiseCoeffs: null
            });

            // Now that the chart instance exists, calculate the initial state and draw it.
            this.calculateAndRedraw();

            this.$watch('renderOriginal', () => this.prepareAndRedraw(false));
            this.$watch('renderSeries', () => this.prepareAndRedraw(false));
            this.$watch('terms_n', () => this.prepareAndRedraw(false));

            this.$watch('calculationMode', (newMode) => {
                if (newMode === 'coefficients') {
                    // When switching to coefficient mode, hide the original function
                    this.renderOriginal = [];
                } else {
                    // When switching back to calculate mode, show it again by default
                    this.renderOriginal = ['original'];
                }
                // Redraw the chart with the new settings
                this.prepareAndRedraw(false);
            });
        },

        addFunction() {
            const lastFunc = this.functions[this.functions.length - 1];
            this.functions.push({
                id: this.nextId++,
                definition: '0',
                domainStart: lastFunc.domainEnd, // Auto-fill for continuity
                domainEnd: '', // User needs to fill this
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            });
        },

        removeFunction(id: number) {
            if (this.functions.length > 1) {
                const funcIndex = this.functions.findIndex(f => f.id === id);
                if (funcIndex > -1) {
                    // If we remove a function in the middle, update the next one's start
                    if (funcIndex > 0 && funcIndex < this.functions.length - 1) {
                        const prevFunc = this.functions[funcIndex - 1];
                        const nextFunc = this.functions[funcIndex + 1];
                        if (nextFunc) {
                            nextFunc.domainStart = prevFunc.domainEnd;
                        }
                    }
                    this.functions = this.functions.filter(f => f.id !== id);
                }
            }
        },

        validate() {
            this.errorMessage = '';
            let hasError = false;

            // Reset all errors first
            this.functions.forEach(func => {
                func.definitionError = null;
                func.domainStartError = null;
                func.domainEndError = null;
            });

            if (this.functions.length === 0) {
                this.errorMessage = "Debe haber al menos una función definida.";
                return false;
            }

            for (let i = 0; i < this.functions.length; i++) {
                const func = this.functions[i];

                const funcValidation = validateFunction(func.definition);
                if (!funcValidation.isValid) {
                    func.definitionError = funcValidation.error!;
                    hasError = true;
                }

                const startValidation = validateConstant(func.domainStart);
                if (!startValidation.isValid) {
                    func.domainStartError = startValidation.error!;
                    hasError = true;
                }

                const endValidation = validateConstant(func.domainEnd);
                if (!endValidation.isValid) {
                    func.domainEndError = endValidation.error!;
                    hasError = true;
                }

                // Continuity validation
                if (i > 0) {
                    const prevFunc = this.functions[i - 1];
                    try {
                        // Only validate continuity if both domain strings are valid constants
                        if (validateConstant(prevFunc.domainEnd).isValid && validateConstant(func.domainStart).isValid) {
                            const prevEnd = math.evaluate(prevFunc.domainEnd);
                            const currentStart = math.evaluate(func.domainStart);
                            if (prevEnd !== currentStart) {
                                hasError = true;
                                const errorMsg = `Debe coincidir con el dominio anterior (${prevEnd})`;
                                func.domainStartError = func.domainStartError ? `${func.domainStartError}. ${errorMsg}`: errorMsg;
                            }
                        }
                    } catch (e) {
                        // Errors will be caught by individual constant validation
                    }
                }
            }

            return !hasError;
        },

        async calculateAndRedraw() {
            this.isLoading = true;
            this.errorMessage = '';
            window.FourierSeriesChart.resetScales();

            if (!this.validate()) {
                this.isLoading = false;
                return;
            }

            try {
                if (this.calculationMode === 'calculate') {
                    const result = calculateCoefficients(this.functions);
                    if (result.success) {
                        this.piecewiseCoeffs = result.coeffs;
                    } else {
                        throw new Error(result.error || 'Ocurrió un error desconocido.');
                    }
                } else { // 'coefficients' mode
                    this.evaluateCoefficientExpressions();
                    if (this.errorMessage) { // if evaluate had an error
                        this.isLoading = false;
                        return;
                    }
                }
                // We have new coefficients, just redraw.
                window.FourierSeriesChart.redraw(this.getChartData());
            } catch (error: any) {
                console.error('[Alpine] Calculation Error:', error);
                this.errorMessage = error.message;
                this.piecewiseCoeffs = null; // Clear coefficients on error
                window.FourierSeriesChart.redraw(this.getChartData()); // Redraw to clear the series
            } finally {
                this.isLoading = false;
            }
        },

        evaluateCoefficientExpressions() {
            try {
                this.errorMessage = '';
                if (this.functions.length === 0) {
                    this.piecewiseCoeffs = null;
                    return;
                };

                const domainStart = math.evaluate(this.functions[0].domainStart);
                const domainEnd = math.evaluate(this.functions[this.functions.length - 1].domainEnd);
                const period = domainEnd - domainStart;

                if (period <= 0) throw new Error("El período total debe ser positivo.");

                const a0_val = math.evaluate(this.coeff_a0_str);
                const an_expr = math.parse(this.coeff_an_str).compile();
                const bn_expr = math.parse(this.coeff_bn_str).compile();

                const an_vals: number[] = [];
                const bn_vals: number[] = [];
                for (let n = 1; n <= 50; n++) { // Using hardcoded 50 like fourier.ts
                    an_vals[n] = an_expr.evaluate({ n });
                    bn_vals[n] = bn_expr.evaluate({ n });
                }

                this.piecewiseCoeffs = {
                    a0: a0_val,
                    an: an_vals,
                    bn: bn_vals,
                    period: period,
                    domainStart: domainStart,
                };

            } catch (e: any) {
                this.errorMessage = `Error en la expresión del coeficiente: ${e.message}`;
                this.piecewiseCoeffs = null;
            }
        },

        prepareAndRedraw(recalculate = true) {
            if (recalculate) {
                this.calculateAndRedraw();
                return;
            }
            // This is called by watchers, just redraw with existing data.
            window.FourierSeriesChart.redraw(this.getChartData());
        },

        getChartData(): FourierChartData {
            return {
                functions: this.functions,
                renderOriginal: this.renderOriginal,
                renderSeries: this.renderSeries,
                terms_n: this.terms_n,
                piecewiseCoeffs: this.piecewiseCoeffs,
            };
        },
    } as unknown as FourierState;
}

// Extender la interfaz global de Window para incluir nuestra función
declare global {
    interface Window {
        fourierState: () => FourierState;
    }
}

window.fourierState = fourierState;