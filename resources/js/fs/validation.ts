import * as math from 'mathjs';

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
        const result = math.evaluate(constStr);
        if (typeof result === 'function' || (typeof result === 'object' && result.isFunction)) {
             return { isValid: false, error: 'La expresión debe ser un valor constante.' };
        }
        return { isValid: true };
    } catch (e: any) {
        return { isValid: false, error: `Expresión inválida: ${e.message}` };
    }
}

/**
 * Validates a string as a mathematical function of 't'.
 * @param funcStr The string to validate.
 * @returns An object with `isValid` and an optional `error` message.
 */
export function validateFunction(funcStr: string): { isValid: boolean; error?: string } {
     if (!funcStr.trim()) {
        return { isValid: false, error: 'La función no puede estar vacía.' };
    }
    try {
        const node = math.parse(funcStr);
        const symbols = node.filter(n => n.isSymbolNode).map(n => n.name);

        // Check for unknown symbols other than 't' and allowed constants
        const allowedSymbols = new Set(['t', 'pi', 'e']);
        const unknownSymbols = symbols.filter(s => !allowedSymbols.has(s) && !(s in math));
        if (unknownSymbols.length > 0) {
            return { isValid: false, error: `Símbolo desconocido: ${unknownSymbols.join(', ')}` };
        }

        // It should be a function of 't' or a constant function
        // We can try to evaluate with a dummy 't' to see if it works
        node.compile().evaluate({ t: 1 });

        return { isValid: true };
    } catch (e: any) {
        return { isValid: false, error: `Función inválida: ${e.message}` };
    }
}
