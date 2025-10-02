import * as math from 'mathjs';
import { validateConstant, validateFunction } from '../utils/validation';
import {
    calculateFourierTransform,
    generateTimeDomainData,
    type CompiledFunction
} from './calculations';

// --- Interfaces ---

interface FunctionInput {
    id: number;
    definition: string;
    domainStart: string;
    domainEnd: string;
    definitionError: string | null;
    domainStartError: string | null;
    domainEndError: string | null;
}

interface FourierTransformState {
    // UI State
    isLoading: boolean;
    errorMessage: string;
    resizeObserver: ResizeObserver | null;

    // Visualization Options
    renderOptions: string[];  // ['time', 'magnitude', 'phase']
    omegaRange: number;
    samplingResolution: number;

    // Function Inputs
    functions: FunctionInput[];
    nextId: number;

    // Methods
    init(): void;
    addFunction(): void;
    removeFunction(id: number): void;
    validate(): boolean;
    calculateTransform(): Promise<void>;
    $watch: (property: string, callback: (value: any) => void) => void;
}

// --- State Implementation ---

function fourierTransformState(): FourierTransformState {
    return {
        // UI State
        isLoading: false,
        errorMessage: '',
        resizeObserver: null,

        // Visualization Options
        renderOptions: ['time', 'magnitude', 'phase'],
        omegaRange: 20,
        samplingResolution: 500,

        // Function Inputs
        nextId: 2,
        functions: [
            {
                id: 1,
                definition: 'exp(-abs(t))',
                domainStart: '-5',
                domainEnd: '5',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            }
        ],

        // --- Methods ---
        init() {
            console.log('[Alpine] Initializing fourierTransformState...');

            // Initialize all charts
            window.TimeDomainChart.init();
            window.MagnitudeChart.init();
            window.PhaseChart.init();

            // Setup ResizeObserver for responsive charts
            const containers = [
                document.getElementById('timeDomainContainer'),
                document.getElementById('magnitudeContainer'),
                document.getElementById('phaseContainer')
            ];

            containers.forEach(container => {
                if (container) {
                    const observer = new ResizeObserver(() => {
                        if (window.TimeDomainChart.chart) window.TimeDomainChart.chart.resize();
                        if (window.MagnitudeChart.chart) window.MagnitudeChart.chart.resize();
                        if (window.PhaseChart.chart) window.PhaseChart.chart.resize();
                    });
                    observer.observe(container);
                    console.log('[Alpine] ResizeObserver attached to container');
                }
            });

            // Watchers for reactive updates
            this.$watch('renderOptions', () => {
                console.log('[Alpine] Render options changed:', this.renderOptions);
                // Future: Toggle chart visibility
            });

            this.$watch('omegaRange', () => {
                console.log('[Alpine] Omega range changed:', this.omegaRange);
                // Future: Recalculate if data exists
            });
        },

        addFunction() {
            const lastFunc = this.functions[this.functions.length - 1];
            this.functions.push({
                id: this.nextId++,
                definition: '0',
                domainStart: lastFunc.domainEnd,
                domainEnd: '',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            });
        },

        removeFunction(id: number) {
            if (this.functions.length > 1) {
                const funcIndex = this.functions.findIndex(f => f.id === id);
                if (funcIndex > -1) {
                    // Update next function's start if needed
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

            // Reset all function errors
            this.functions.forEach(func => {
                func.definitionError = null;
                func.domainStartError = null;
                func.domainEndError = null;
            });

            if (this.functions.length === 0) {
                this.errorMessage = "Debe haber al menos una función definida.";
                return false;
            }

            // Validate each function
            for (let i = 0; i < this.functions.length; i++) {
                const func = this.functions[i];

                const funcValidation = validateFunction(func.definition, 't');
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
                        if (validateConstant(prevFunc.domainEnd).isValid && validateConstant(func.domainStart).isValid) {
                            const prevEnd = math.evaluate(prevFunc.domainEnd);
                            const currentStart = math.evaluate(func.domainStart);
                            if (prevEnd !== currentStart) {
                                hasError = true;
                                const errorMsg = `Debe coincidir con el dominio anterior (${prevEnd})`;
                                func.domainStartError = func.domainStartError ? `${func.domainStartError}. ${errorMsg}` : errorMsg;
                            }
                        }
                    } catch (e) {
                        // Errors caught by individual validation
                    }
                }
            }

            return !hasError;
        },

        async calculateTransform() {
            this.isLoading = true;
            this.errorMessage = '';

            if (!this.validate()) {
                this.isLoading = false;
                return;
            }

            try {
                console.log('[Alpine] Calculating Fourier Transform...');

                // 1. Compile functions
                const compiledFunctions: CompiledFunction[] = this.functions.map(func => ({
                    compiled: math.parse(func.definition).compile(),
                    domainStart: math.evaluate(func.domainStart),
                    domainEnd: math.evaluate(func.domainEnd)
                }));

                // 2. Generate time domain data for plotting
                const timeDomainData = generateTimeDomainData(compiledFunctions, 500);

                // 3. Calculate Fourier Transform
                const ftResult = calculateFourierTransform(
                    compiledFunctions,
                    this.omegaRange,
                    this.samplingResolution
                );

                if (!ftResult.success) {
                    throw new Error(ftResult.error || 'Error en el cálculo de la transformada');
                }

                // 4. Render charts based on selected options
                if (this.renderOptions.includes('time')) {
                    window.TimeDomainChart.redraw({
                        t: timeDomainData.t,
                        values: timeDomainData.values
                    });
                }

                if (this.renderOptions.includes('magnitude')) {
                    window.MagnitudeChart.redraw({
                        omega: ftResult.omega,
                        magnitude: ftResult.magnitude
                    });
                }

                if (this.renderOptions.includes('phase')) {
                    window.PhaseChart.redraw({
                        omega: ftResult.omega,
                        phase: ftResult.phase
                    });
                }

                console.log('[Alpine] Fourier Transform calculation completed successfully');

            } catch (error: any) {
                console.error('[Alpine] Calculation Error:', error);
                this.errorMessage = error.message;
            } finally {
                this.isLoading = false;
            }
        },

    } as unknown as FourierTransformState;
}

// --- Global Declaration ---
declare global {
    interface Window {
        fourierTransformState: () => FourierTransformState;
    }
}

window.fourierTransformState = fourierTransformState;