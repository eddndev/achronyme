/**
 * Convolution Calculations
 *
 * Implements continuous convolution for piecewise functions
 * using numerical integration (Simpson's Rule).
 */

import { type EvalFunction } from 'mathjs';
import { integrateNumerically } from '../utils/numerical-integration';

// --- Interfaces ---

/**
 * Compiled piecewise function segment
 */
export interface CompiledFunction {
    compiled: EvalFunction;
    domainStart: number;
    domainEnd: number;
}

/**
 * Result of Convolution calculation
 */
export interface ConvolutionResult {
    success: boolean;
    t: number[];           // Time array
    result: number[];      // (f*g)(t) values
    error?: string;
}

/**
 * Function domain data for plotting
 */
export interface FunctionData {
    tau: number[];
    values: number[];
}

// --- Main Convolution Calculation ---

/**
 * Calculate Continuous Convolution using numerical integration
 *
 * Formula: (f * g)(t) = ∫_{-∞}^{∞} f(τ) · g(t - τ) dτ
 *
 * For piecewise functions with limited support:
 * Integration is performed only over the regions where both functions are non-zero
 *
 * @param functionsF - Array of compiled piecewise functions for f(t)
 * @param functionsG - Array of compiled piecewise functions for g(t)
 * @param tRange - [tMin, tMax] range to evaluate convolution
 * @param resolution - Number of points to calculate
 * @returns Convolution result
 */
export function calculateConvolution(
    functionsF: CompiledFunction[],
    functionsG: CompiledFunction[],
    tRange: [number, number],
    resolution: number
): ConvolutionResult {
    try {
        if (!functionsF || functionsF.length === 0) {
            throw new Error("No se proporcionaron funciones f(t) para el cálculo.");
        }

        if (!functionsG || functionsG.length === 0) {
            throw new Error("No se proporcionaron funciones g(t) para el cálculo.");
        }

        const [tMin, tMax] = tRange;

        if (tMax <= tMin) {
            throw new Error("El rango de tiempo debe ser válido (tMax > tMin).");
        }

        if (resolution < 10) {
            throw new Error("La resolución debe ser al menos 10 puntos.");
        }

        const t: number[] = [];
        const result: number[] = [];

        const tStep = (tMax - tMin) / resolution;

        // Determine overall domain for f and g
        const fDomain = {
            start: functionsF[0].domainStart,
            end: functionsF[functionsF.length - 1].domainEnd
        };

        const gDomain = {
            start: functionsG[0].domainStart,
            end: functionsG[functionsG.length - 1].domainEnd
        };

        // Helper function to evaluate piecewise f(τ)
        const evaluateF = (tau: number): number => {
            for (const func of functionsF) {
                if (tau >= func.domainStart - 1e-9 && tau <= func.domainEnd + 1e-9) {
                    try {
                        const value = func.compiled.evaluate({ t: tau });
                        return isFinite(value) ? value : 0;
                    } catch (e) {
                        return 0;
                    }
                }
            }
            return 0;
        };

        // Helper function to evaluate piecewise g(t - τ)
        const evaluateG = (t_minus_tau: number): number => {
            for (const func of functionsG) {
                if (t_minus_tau >= func.domainStart - 1e-9 && t_minus_tau <= func.domainEnd + 1e-9) {
                    try {
                        const value = func.compiled.evaluate({ t: t_minus_tau });
                        return isFinite(value) ? value : 0;
                    } catch (e) {
                        return 0;
                    }
                }
            }
            return 0;
        };

        // Calculate convolution for each time point
        for (let time = tMin; time <= tMax; time += tStep) {
            t.push(time);

            // Determine integration limits
            // For f(τ) * g(t-τ), we need:
            // - τ in domain of f: [fStart, fEnd]
            // - t-τ in domain of g: τ in [t - gEnd, t - gStart]
            // So τ should be in the intersection

            const tauMin = Math.max(fDomain.start, time - gDomain.end);
            const tauMax = Math.min(fDomain.end, time - gDomain.start);

            let convValue = 0;

            if (tauMin < tauMax) {
                // Integrand: f(τ) * g(t - τ)
                const integrand = (tau: number) => {
                    const fVal = evaluateF(tau);
                    const gVal = evaluateG(time - tau);
                    return fVal * gVal;
                };

                convValue = integrateNumerically(
                    integrand,
                    tauMin,
                    tauMax,
                    1000 // Integration steps for accuracy
                );
            }

            result.push(convValue);
        }

        console.log(`[Convolution] Calculated ${t.length} points`);
        console.log(`[Convolution] t range: [${t[0].toFixed(2)}, ${t[t.length-1].toFixed(2)}]`);

        return {
            success: true,
            t,
            result
        };

    } catch (error: any) {
        console.error('[Convolution] Calculation error:', error);
        return {
            success: false,
            t: [],
            result: [],
            error: error.message
        };
    }
}

/**
 * Calculate automatic convolution range based on function domains
 *
 * The convolution range is [a_f + a_g, b_f + b_g]
 *
 * @param functionsF - Functions f(t)
 * @param functionsG - Functions g(t)
 * @returns [tMin, tMax] range
 */
export function calculateAutomaticRange(
    functionsF: CompiledFunction[],
    functionsG: CompiledFunction[]
): [number, number] {
    const fStart = functionsF[0].domainStart;
    const fEnd = functionsF[functionsF.length - 1].domainEnd;

    const gStart = functionsG[0].domainStart;
    const gEnd = functionsG[functionsG.length - 1].domainEnd;

    const tMin = fStart + gStart;
    const tMax = fEnd + gEnd;

    return [tMin, tMax];
}

/**
 * Generate time domain samples for function f(t)
 *
 * @param functions - Compiled piecewise functions
 * @param numPoints - Number of points to sample
 * @returns Time domain data for plotting
 */
export function generateFunctionData(
    functions: CompiledFunction[],
    numPoints: number = 500
): FunctionData {
    if (!functions || functions.length === 0) {
        return { tau: [], values: [] };
    }

    const tau: number[] = [];
    const values: number[] = [];

    const tauStart = functions[0].domainStart;
    const tauEnd = functions[functions.length - 1].domainEnd;
    const tauStep = (tauEnd - tauStart) / numPoints;

    for (let t = tauStart; t <= tauEnd; t += tauStep) {
        tau.push(t);

        let value = 0;
        for (const func of functions) {
            if (t >= func.domainStart - 1e-9 && t <= func.domainEnd + 1e-9) {
                try {
                    value = func.compiled.evaluate({ t });
                    if (!isFinite(value)) {
                        value = 0;
                    }
                } catch (e) {
                    value = 0;
                }
                break;
            }
        }

        values.push(value);
    }

    return { tau, values };
}

/**
 * Generate g(t-τ) for a specific time t
 *
 * This is g shifted and flipped for visualization
 *
 * @param functionsG - Compiled functions for g
 * @param currentTime - Current time t
 * @param tauRange - Range of τ to evaluate
 * @param numPoints - Number of points
 * @returns Function data for g(t-τ)
 */
export function generateShiftedG(
    functionsG: CompiledFunction[],
    currentTime: number,
    tauRange: [number, number],
    numPoints: number = 500
): FunctionData {
    const tau: number[] = [];
    const values: number[] = [];

    const [tauStart, tauEnd] = tauRange;
    const tauStep = (tauEnd - tauStart) / numPoints;

    for (let t = tauStart; t <= tauEnd; t += tauStep) {
        tau.push(t);

        const shiftedArg = currentTime - t; // t - τ
        let value = 0;

        for (const func of functionsG) {
            if (shiftedArg >= func.domainStart - 1e-9 && shiftedArg <= func.domainEnd + 1e-9) {
                try {
                    value = func.compiled.evaluate({ t: shiftedArg });
                    if (!isFinite(value)) {
                        value = 0;
                    }
                } catch (e) {
                    value = 0;
                }
                break;
            }
        }

        values.push(value);
    }

    return { tau, values };
}

/**
 * Calculate product f(τ) * g(t-τ) for visualization
 *
 * @param fData - f(τ) data
 * @param gData - g(t-τ) data
 * @returns Product values
 */
export function calculateProduct(
    fData: FunctionData,
    gData: FunctionData
): number[] {
    const minLength = Math.min(fData.values.length, gData.values.length);
    const product: number[] = [];

    for (let i = 0; i < minLength; i++) {
        product.push(fData.values[i] * gData.values[i]);
    }

    return product;
}