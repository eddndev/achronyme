import { evaluate, parse } from 'mathjs';

// Common mathematical functions and constants available in mathjs
const MATH_SYMBOLS = new Set([
    'abs', 'acos', 'asin', 'atan', 'atan2', 'ceil', 'cos', 'exp', 'floor',
    'log', 'log10', 'max', 'min', 'pow', 'random', 'round', 'sin', 'sqrt',
    'tan', 'pi', 'e', 'i', 'sign', 'sinh', 'cosh', 'tanh', 'rect', 'sinc'
]);

/**
 * Validates a string as a mathematical expression that can be evaluated to a constant.
 * @param constStr The string to validate.
 * @returns An object with `isValid` and an optional `error` message.
 */
export function validateConstant(constStr: string): { isValid: boolean; error?: string } {
    if (!constStr.trim()) {
        return { isValid: false, error: 'El campo no puede estar vacío.' };
    }
    try {
        const result = evaluate(constStr);
        if (typeof result === 'function' || (typeof result === 'object' && result.isFunction)) {
             return { isValid: false, error: 'La expresión debe ser un valor constante.' };
        }
        return { isValid: true };
    } catch (e: any) {
        return { isValid: false, error: `Expresión inválida: ${e.message}` };
    }
}

/**
 * Validates a string as a mathematical function.
 * @param funcStr The string to validate.
 * @param domainVar The domain variable (default: 't'). Can be 'n', 'x', 'omega', etc.
 * @returns An object with `isValid` and an optional `error` message.
 */
export function validateFunction(
    funcStr: string,
    domainVar: string = 't'
): { isValid: boolean; error?: string } {
     if (!funcStr.trim()) {
        return { isValid: false, error: 'La función no puede estar vacía.' };
    }
    try {
        const node = parse(funcStr);
        const symbols = node.filter(n => n.isSymbolNode).map(n => n.name);

        // Check for unknown symbols other than the domain variable and allowed constants
        const allowedSymbols = new Set([domainVar, 'pi', 'e', 'i']);
        const unknownSymbols = symbols.filter(s => !allowedSymbols.has(s) && !MATH_SYMBOLS.has(s));
        if (unknownSymbols.length > 0) {
            return { isValid: false, error: `Símbolo desconocido: ${unknownSymbols.join(', ')}` };
        }

        // It should be a function of the domain variable or a constant function
        // Try to evaluate with a dummy value to see if it works
        const testScope = { [domainVar]: 1 };
        node.compile().evaluate(testScope);

        return { isValid: true };
    } catch (e: any) {
        return { isValid: false, error: `Función inválida: ${e.message}` };
    }
}