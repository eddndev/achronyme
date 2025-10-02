import { parse, evaluate } from 'mathjs';
import { integrateNumerically } from '../utils/numerical-integration';

const MAX_TERMS = 50;

// --- Interfaces ---
// To avoid dependency cycles, we define the needed shape here.
interface FunctionInput {
    definition: string;
    domainStart: string;
    domainEnd: string;
}

// --- Main Coefficient Calculation for Piecewise Functions ---
export function calculateCoefficients(
    functions: FunctionInput[]
) {
    try {
        if (!functions || functions.length === 0) {
            throw new Error("No se proporcionaron funciones para el cálculo.");
        }

        // 1. Compile functions and evaluate domains
        const compiledFuncs = functions.map(f => {
            try {
                return {
                    userFunc: parse(f.definition).compile(),
                    a: evaluate(f.domainStart),
                    b: evaluate(f.domainEnd)
                };
            } catch (e: any) {
                throw new Error(`Error al procesar la función "${f.definition}" o sus dominios: ${e.message}`);
            }
        });

        // 2. Determine total period
        const totalDomainStart = compiledFuncs[0].a;
        const totalDomainEnd = compiledFuncs[compiledFuncs.length - 1].b;
        const period = totalDomainEnd - totalDomainStart;

        if (period <= 0) {
            throw new Error("El período total (desde el inicio del primer dominio hasta el fin del último) debe ser positivo.");
        }

        // 3. Calculate coefficients by summing integrals over each piece
        const an: number[] = [];
        const bn: number[] = [];

        // Calculate a0
        let integral_a0 = 0;
        for (const f of compiledFuncs) {
            integral_a0 += integrateNumerically(t => f.userFunc.evaluate({ t }), f.a, f.b);
        }
        const a0 = (2 / period) * integral_a0;

        // Calculate an and bn for n=1 to MAX_TERMS
        for (let n = 1; n <= MAX_TERMS; n++) {
            let integral_an = 0;
            let integral_bn = 0;

            for (const f of compiledFuncs) {
                const integrand_an = (t: number) => f.userFunc.evaluate({ t }) * Math.cos(2 * Math.PI * n * t / period);
                integral_an += integrateNumerically(integrand_an, f.a, f.b);

                const integrand_bn = (t: number) => f.userFunc.evaluate({ t }) * Math.sin(2 * Math.PI * n * t / period);
                integral_bn += integrateNumerically(integrand_bn, f.a, f.b);
            }
            an[n] = (2 / period) * integral_an;
            bn[n] = (2 / period) * integral_bn;
        }

        return {
            success: true,
            coeffs: { a0, an, bn, period, domainStart: totalDomainStart },
        };

    } catch (error: any) {
        console.error("Error en el cálculo de coeficientes:", error);
        return { success: false, error: error.message };
    }
}