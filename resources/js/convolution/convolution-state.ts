import * as math from 'mathjs';
import { validateConstant, validateFunction } from '../utils/validation';
import {
    calculateConvolution,
    calculateAutomaticRange,
    generateFunctionData,
    generateShiftedG,
    calculateProduct,
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

interface ConvolutionState {
    // UI State
    isLoading: boolean;
    errorMessage: string;
    resizeObserver: ResizeObserver | null;

    // Visualization Options
    renderOptions: string[];  // ['f', 'g', 'product', 'result']
    currentTime: number;
    manualRange: boolean;

    // Range Parameters
    tInitial: string;
    tInitial_error: string | null;
    tFinal: string;
    tFinal_error: string | null;

    // Computed range values for slider
    get tInitialNumeric(): number;
    get tFinalNumeric(): number;

    // Function Inputs
    functionsF: FunctionInput[];
    functionsG: FunctionInput[];
    nextIdF: number;
    nextIdG: number;

    // Cached calculation data
    cachedData: {
        compiledF: CompiledFunction[] | null;
        compiledG: CompiledFunction[] | null;
        convResult: any | null;
        fData: any | null;
        tauMin: number | null;
        tauMax: number | null;
        tRange: [number, number] | null;
    };

    // Methods
    init(): void;
    addFunctionF(): void;
    removeFunctionF(id: number): void;
    addFunctionG(): void;
    removeFunctionG(id: number): void;
    validate(): boolean;
    calculateConvolution(): Promise<void>;
    updateCharts(): void;
    $watch: (property: string, callback: (value: any) => void) => void;
}

// --- State Implementation ---

function convolutionState(): ConvolutionState {
    return {
        // UI State
        isLoading: false,
        errorMessage: '',
        resizeObserver: null,

        // Visualization Options
        renderOptions: ['f', 'g', 'product', 'result'],
        currentTime: 0,
        manualRange: false,

        // Range Parameters
        tInitial: '-10',
        tInitial_error: null,
        tFinal: '10',
        tFinal_error: null,

        // Function Inputs - f(t)
        nextIdF: 2,
        functionsF: [
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

        // Function Inputs - g(t)
        nextIdG: 2,
        functionsG: [
            {
                id: 1,
                definition: '1',
                domainStart: '-1',
                domainEnd: '1',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            }
        ],

        // Cached calculation data
        cachedData: {
            compiledF: null,
            compiledG: null,
            convResult: null,
            fData: null,
            tauMin: null,
            tauMax: null,
            tRange: null
        },

        // --- Computed Properties ---
        get tInitialNumeric(): number {
            try {
                return math.evaluate(this.tInitial);
            } catch (e) {
                return -10; // Fallback
            }
        },

        get tFinalNumeric(): number {
            try {
                return math.evaluate(this.tFinal);
            } catch (e) {
                return 10; // Fallback
            }
        },

        // --- Methods ---
        init() {
            console.log('[Alpine] Initializing convolutionState...');

            // Initialize all charts
            window.FunctionsChart.init();
            window.ResultChart.init();

            // Setup ResizeObserver for responsive charts
            const containers = [
                document.getElementById('functionsContainer'),
                document.getElementById('resultContainer')
            ];

            containers.forEach(container => {
                if (container) {
                    const observer = new ResizeObserver(() => {
                        if (window.FunctionsChart.chart) window.FunctionsChart.chart.resize();
                        if (window.ResultChart.chart) window.ResultChart.chart.resize();
                    });
                    observer.observe(container);
                    console.log('[Alpine] ResizeObserver attached to container');
                }
            });

            // Watchers for reactive updates
            this.$watch('renderOptions', () => {
                console.log('[Alpine] Render options changed:', this.renderOptions);
                this.updateCharts();
            });

            this.$watch('currentTime', () => {
                console.log('[Alpine] Current time changed:', this.currentTime);
                this.updateCharts();
            });

            this.$watch('manualRange', () => {
                console.log('[Alpine] Manual range:', this.manualRange);
            });

            // Calculate convolution on initialization with default functions
            this.calculateConvolution();
        },

        addFunctionF() {
            const lastFunc = this.functionsF[this.functionsF.length - 1];
            this.functionsF.push({
                id: this.nextIdF++,
                definition: '0',
                domainStart: lastFunc.domainEnd,
                domainEnd: '',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            });
        },

        removeFunctionF(id: number) {
            if (this.functionsF.length > 1) {
                const funcIndex = this.functionsF.findIndex(f => f.id === id);
                if (funcIndex > -1) {
                    if (funcIndex > 0 && funcIndex < this.functionsF.length - 1) {
                        const prevFunc = this.functionsF[funcIndex - 1];
                        const nextFunc = this.functionsF[funcIndex + 1];
                        if (nextFunc) {
                            nextFunc.domainStart = prevFunc.domainEnd;
                        }
                    }
                    this.functionsF = this.functionsF.filter(f => f.id !== id);
                }
            }
        },

        addFunctionG() {
            const lastFunc = this.functionsG[this.functionsG.length - 1];
            this.functionsG.push({
                id: this.nextIdG++,
                definition: '0',
                domainStart: lastFunc.domainEnd,
                domainEnd: '',
                definitionError: null,
                domainStartError: null,
                domainEndError: null,
            });
        },

        removeFunctionG(id: number) {
            if (this.functionsG.length > 1) {
                const funcIndex = this.functionsG.findIndex(f => f.id === id);
                if (funcIndex > -1) {
                    if (funcIndex > 0 && funcIndex < this.functionsG.length - 1) {
                        const prevFunc = this.functionsG[funcIndex - 1];
                        const nextFunc = this.functionsG[funcIndex + 1];
                        if (nextFunc) {
                            nextFunc.domainStart = prevFunc.domainEnd;
                        }
                    }
                    this.functionsG = this.functionsG.filter(f => f.id !== id);
                }
            }
        },

        validate() {
            this.errorMessage = '';
            let hasError = false;

            // Reset all function errors for f(t)
            this.functionsF.forEach(func => {
                func.definitionError = null;
                func.domainStartError = null;
                func.domainEndError = null;
            });

            // Reset all function errors for g(t)
            this.functionsG.forEach(func => {
                func.definitionError = null;
                func.domainStartError = null;
                func.domainEndError = null;
            });

            // Validate f(t) functions
            if (this.functionsF.length === 0) {
                this.errorMessage = "Debe haber al menos una función f(t) definida.";
                return false;
            }

            for (let i = 0; i < this.functionsF.length; i++) {
                const func = this.functionsF[i];

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
                    const prevFunc = this.functionsF[i - 1];
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

            // Validate g(t) functions
            if (this.functionsG.length === 0) {
                this.errorMessage = "Debe haber al menos una función g(t) definida.";
                return false;
            }

            for (let i = 0; i < this.functionsG.length; i++) {
                const func = this.functionsG[i];

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
                    const prevFunc = this.functionsG[i - 1];
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

            // Validate range if manual
            if (this.manualRange) {
                this.tInitial_error = null;
                this.tFinal_error = null;

                const tInitialValidation = validateConstant(this.tInitial);
                if (!tInitialValidation.isValid) {
                    this.tInitial_error = tInitialValidation.error!;
                    hasError = true;
                }

                const tFinalValidation = validateConstant(this.tFinal);
                if (!tFinalValidation.isValid) {
                    this.tFinal_error = tFinalValidation.error!;
                    hasError = true;
                }

                // Check that tFinal > tInitial
                if (tInitialValidation.isValid && tFinalValidation.isValid) {
                    const tInit = math.evaluate(this.tInitial);
                    const tFin = math.evaluate(this.tFinal);
                    if (tFin <= tInit) {
                        this.tFinal_error = 't final debe ser mayor que t inicial';
                        hasError = true;
                    }
                }
            }

            return !hasError;
        },

        async calculateConvolution() {
            this.isLoading = true;
            this.errorMessage = '';

            if (!this.validate()) {
                this.isLoading = false;
                return;
            }

            try {
                console.log('[Alpine] Calculating Convolution...');

                // 1. Compile functions f(t)
                const compiledF: CompiledFunction[] = this.functionsF.map(func => ({
                    compiled: math.parse(func.definition).compile(),
                    domainStart: math.evaluate(func.domainStart),
                    domainEnd: math.evaluate(func.domainEnd)
                }));

                // 2. Compile functions g(t)
                const compiledG: CompiledFunction[] = this.functionsG.map(func => ({
                    compiled: math.parse(func.definition).compile(),
                    domainStart: math.evaluate(func.domainStart),
                    domainEnd: math.evaluate(func.domainEnd)
                }));

                // 3. Determine time range for convolution
                let tRange: [number, number];

                if (this.manualRange) {
                    tRange = [
                        math.evaluate(this.tInitial),
                        math.evaluate(this.tFinal)
                    ];
                } else {
                    tRange = calculateAutomaticRange(compiledF, compiledG);
                    // Update slider range for animation
                    this.tInitial = tRange[0].toString();
                    this.tFinal = tRange[1].toString();
                }

                console.log(`[Alpine] Time range: [${tRange[0]}, ${tRange[1]}]`);

                // 4. Calculate convolution
                const convResult = calculateConvolution(
                    compiledF,
                    compiledG,
                    tRange,
                    500 // Fixed sampling resolution
                );

                if (!convResult.success) {
                    throw new Error(convResult.error || 'Error en el cálculo de la convolución');
                }

                // 5. Generate data for functions chart
                const fData = generateFunctionData(compiledF, 500);

                // Determine tau range for visualization
                const tauMin = Math.min(fData.tau[0], compiledG[0].domainStart);
                const tauMax = Math.max(fData.tau[fData.tau.length - 1], compiledG[compiledG.length - 1].domainEnd);

                // Cache all calculation data
                this.cachedData = {
                    compiledF,
                    compiledG,
                    convResult,
                    fData,
                    tauMin,
                    tauMax,
                    tRange
                };

                // 6. Set currentTime to middle of range
                const midPoint = (tRange[0] + tRange[1]) / 2;
                this.currentTime = Math.round(midPoint * 10) / 10; // Round to 1 decimal

                // 7. Update charts
                this.updateCharts();

                console.log('[Alpine] Convolution calculation completed successfully');

            } catch (error: any) {
                console.error('[Alpine] Calculation Error:', error);
                this.errorMessage = error.message;
            } finally {
                this.isLoading = false;
            }
        },

        updateCharts() {
            // Only update if we have cached data
            if (!this.cachedData.fData || !this.cachedData.convResult) {
                return;
            }

            const { compiledG, fData, tauMin, tauMax, convResult, tRange } = this.cachedData;

            // Determine what to show based on renderOptions
            const showF = this.renderOptions.includes('f');
            const showG = this.renderOptions.includes('g');
            const showProduct = this.renderOptions.includes('product');
            const showResult = this.renderOptions.includes('result');

            // Use current time from slider
            const gData = generateShiftedG(
                compiledG!,
                this.currentTime,
                [tauMin!, tauMax!],
                500
            );

            const productData = calculateProduct(fData, gData);

            // Render functions chart
            window.FunctionsChart.redraw({
                tau: fData.tau,
                f: showF ? fData.values : [],
                g: showG ? gData.values : [],
                product: showProduct ? productData : undefined
            });

            // Render result chart
            if (showResult) {
                // Find the closest point or interpolate
                let interpolatedValue = 0;
                const currentT = this.currentTime;

                // Find the two closest points for interpolation
                let leftIndex = -1;
                for (let i = 0; i < convResult.t.length - 1; i++) {
                    if (convResult.t[i] <= currentT && convResult.t[i + 1] >= currentT) {
                        leftIndex = i;
                        break;
                    }
                }

                if (leftIndex >= 0 && leftIndex < convResult.t.length - 1) {
                    // Linear interpolation
                    const t1 = convResult.t[leftIndex];
                    const t2 = convResult.t[leftIndex + 1];
                    const y1 = convResult.result[leftIndex];
                    const y2 = convResult.result[leftIndex + 1];

                    const ratio = (currentT - t1) / (t2 - t1);
                    interpolatedValue = y1 + ratio * (y2 - y1);
                } else if (currentT <= convResult.t[0]) {
                    interpolatedValue = convResult.result[0];
                } else if (currentT >= convResult.t[convResult.t.length - 1]) {
                    interpolatedValue = convResult.result[convResult.result.length - 1];
                }

                const currentPoint = {
                    t: this.currentTime,
                    value: interpolatedValue
                };

                window.ResultChart.redraw({
                    t: convResult.t,
                    result: convResult.result,
                    currentPoint
                });
            } else {
                // Clear result chart if not showing
                window.ResultChart.clear();
            }
        },

    } as unknown as ConvolutionState;
}

// --- Global Declaration ---
declare global {
    interface Window {
        convolutionState: () => ConvolutionState;
    }
}

window.convolutionState = convolutionState;