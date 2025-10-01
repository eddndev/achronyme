import * as math from 'mathjs';

// 1. Port de la integración numérica (Regla de Simpson)
function integrateNumerically(
    fn: (t: number) => number,
    a: number,
    b: number,
    steps: number = 1000
): number {
    if (steps % 2 !== 0) steps++; // Debe ser par
    const h = (b - a) / steps;
    let sum = fn(a) + fn(b);

    for (let i = 1; i < steps; i++) {
        const x = a + i * h;
        sum += (i % 2 !== 0) ? 4 * fn(x) : 2 * fn(x);
    }
    return (h / 3) * sum;
}

// 2. Función principal para calcular los coeficientes
export function calculateCoefficients(
    functionStr: string,
    domainStartStr: string,
    domainEndStr: string,
    numTerms: number
) {
    try {
        // Evaluar límites y período
        const a = math.evaluate(domainStartStr);
        const b = math.evaluate(domainEndStr);
        const period = b - a;

        if (period <= 0) throw new Error("El período debe ser positivo.");

        // Compilar la función del usuario de forma segura
        const node = math.parse(functionStr);
        const code = node.compile();
        const userFunc = (t: number) => code.evaluate({ t });

        // Calcular a0
        const integral_a0 = integrateNumerically(userFunc, a, b);
        const a0 = (1 / period) * integral_a0;

        // Calcular an y bn
        const an: number[] = [];
        const bn: number[] = [];

        for (let n = 1; n <= numTerms; n++) {
            // Integrando para an
            const integrand_an = (t: number) => userFunc(t) * Math.cos(2 * Math.PI * n * t / period);
            const integral_an = integrateNumerically(integrand_an, a, b);
            an[n] = (2 / period) * integral_an;

            // Integrando para bn
            const integrand_bn = (t: number) => userFunc(t) * Math.sin(2 * Math.PI * n * t / period);
            const integral_bn = integrateNumerically(integrand_bn, a, b);
            bn[n] = (2 / period) * integral_bn;
        }
        
        return {
            success: true,
            coeffs: { a0, an, bn, period, domainStart: a },
        };

    } catch (error) {
        console.error("Error en el cálculo:", error);
        return { success: false, error: error.message };
    }
}
