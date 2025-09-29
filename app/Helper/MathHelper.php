<?php

namespace App\Support;

use Symfony\Component\ExpressionLanguage\ExpressionLanguage;
use Symfony\Component\ExpressionLanguage\ExpressionFunction;

class MathHelper
{
    /** @var ExpressionLanguage|null */
    private static ?ExpressionLanguage $el = null;

    /**
     * Normaliza una función escrita “humana” a una cadena segura para Wolfram Language.
     * Acepta: t, pi/Pi, sin/ cos/ tan/ exp/ log/ abs/ sqrt con paréntesis.
     * Devuelve algo como: "t*Sin[3*t]" listo para embutir en Integrate[…].
     *
     * @throws \InvalidArgumentException
     */
    public static function normalizeFunctionForWolfram(string $s): string
    {
        // 0) espacios
        $s = trim(preg_replace('/\s+/', ' ', $s));

        // 1) funciones en formato nombre( → Nombre[
        //    (añade "sen(" como alias de sin en español)
        $funcs = [
            '/\bsen\s*\(/i'  => 'Sin[',
            '/\bsin\s*\(/i'  => 'Sin[',
            '/\bcos\s*\(/i'  => 'Cos[',
            '/\btan\s*\(/i'  => 'Tan[',
            '/\bexp\s*\(/i'  => 'Exp[',
            '/\blog\s*\(/i'  => 'Log[',
            '/\babs\s*\(/i'  => 'Abs[',
            '/\bsqrt\s*\(/i' => 'Sqrt[',
        ];
        $s = preg_replace(array_keys($funcs), array_values($funcs), $s);

        // 2) pi → Pi (Wolfram)
        $s = preg_replace('/\bpi\b/i', 'Pi', $s);

        // 3) potencias ** → ^
        $s = str_replace('**', '^', $s);

        // 4) cerrar paréntesis de funciones: convierte ')' en ']' mientras haya
        //    funciones abiertas (Sin[, Cos[, …) sin cerrar.
        $s = self::closeFunctionParens($s);

        // 5) validación de caracteres
        if (!preg_match('/^[A-Za-z0-9\+\-\*\/\^\.\,\s\[\]\(\)]+$/', $s)) {
            throw new \InvalidArgumentException('La función contiene caracteres no permitidos.');
        }

        // 6) paréntesis/corchetes balanceados (muy básico)
        if (!self::balancedDelimiters($s)) {
            throw new \InvalidArgumentException('Paréntesis o corchetes desbalanceados.');
        }

        // 7) Variable permitida: solo "t" (opcional pero recomendable)
        if (preg_match('/\b(?!t\b)[a-su-zA-SU-Z]\w*/', $s)) {
            throw new \InvalidArgumentException('Solo se permite la variable t en la función.');
        }

        return trim($s);
    }

    /**
     * Normaliza un límite para Wolfram: acepta números, Pi, + - * / ^ y paréntesis.
     *
     * @throws \InvalidArgumentException
     */
    public static function normalizeLimitForWolfram(string $s): string
    {
        $s = trim($s);
        $s = preg_replace('/\s+/', '', $s);
        $s = preg_replace('/\bpi\b/i', 'Pi', $s);
        $s = str_replace('**', '^', $s);

        if (!preg_match('/^[0-9\+\-\*\/\^\.\(\)Pi]+$/', $s)) {
            throw new \InvalidArgumentException('Límite inválido. Usa números y Pi.');
        }
        if (!self::balancedDelimiters($s)) {
            throw new \InvalidArgumentException('Paréntesis desbalanceados en el límite.');
        }
        return $s;
    }

    /**
     * Convierte una expresión en Wolfram (moutput) a una cadena evaluable por Symfony ExpressionLanguage.
     * Inserta multiplicaciones faltantes (implícitas), normaliza funciones, Pi→pi, ^→**.
     */
    public static function wlToEL(string $s): string
    {
        // 1) mapeos de símbolos/funciones
        $map = [
            'π'     => 'pi',
            'Pi'    => 'pi',
            'Sin['  => 'sin(',
            'Cos['  => 'cos(',
            'Tan['  => 'tan(',
            'Exp['  => 'exp(',
            'Log['  => 'log(',
            'Abs['  => 'abs(',
            ']'     => ')',
            '^'     => '**',
            'E^('   => 'exp(',
        ];
        $s = strtr($s, $map);

        // 2) colapsar espacios
        $s = trim(preg_replace('/\s+/', ' ', $s));

        // 3) multiplicación implícita con ESPACIOS: "n pi", "-2 n", ") n", "(n**2) pi"
        $s = preg_replace('/(?<=\d|\)|[A-Za-z])\s+(?=\d|\(|[A-Za-z])/', '*', $s);

        // 4) multiplicación implícita pegada: "2n"→"2*n", "n2"→"n*2"
        $s = preg_replace('/(\d)([A-Za-z\(])/', '$1*$2', $s);
        $s = preg_replace('/([A-Za-z\)])(\d)/', '$1*$2', $s);

        return trim($s);
    }

    /**
     * Evalúa una expresión EL para n = 1..N.
     * Retorna [1 => val1, 2 => val2, ...].
     */
    public static function evalForNsEL(string $exprEL, int $N = 50): array
    {
        $el = self::getEL();
        $vals = [];
        for ($n = 1; $n <= $N; $n++) {
            $v = $el->evaluate($exprEL, ['n' => $n, 'pi' => \M_PI]);
            // Limpieza de ruido numérico
            $eps = 1e-12;
            if (is_numeric($v)) {
                if (abs($v) < $eps) $v = 0.0;
                if (abs($v - round($v)) < $eps) $v = (float) round($v);
            }
            $vals[$n] = $v;
        }
        return $vals;
    }

    /* =================== Helpers internos =================== */

    /** Cierra ) → ] únicamente para llamadas de función abiertas (Sin[, Cos[, …). */
    private static function closeFunctionParens(string $s): string
    {
        $out   = '';
        $depth = 0; // profundidad de funciones con '['
        $len   = strlen($s);

        for ($i = 0; $i < $len; $i++) {
            $ch = $s[$i];

            if ($ch === '[') {
                $depth++;
                $out .= $ch;
                continue;
            }
            if ($ch === ']') {
                // rara vez viene de entrada, pero la aceptamos
                $depth = max(0, $depth - 1);
                $out .= $ch;
                continue;
            }
            if ($ch === ')') {
                if ($depth > 0) {
                    // cierra una llamada de función
                    $depth--;
                    $out .= ']';
                } else {
                    // paréntesis normal
                    $out .= $ch;
                }
                continue;
            }

            $out .= $ch;
        }

        // si sobran '[' sin cerrar, no intentamos corregir: validación lo atrapará
        return $out;
    }

    /** Verificación básica de balance de () y []. */
    private static function balancedDelimiters(string $s): bool
    {
        $stack = 0;
        $stack2 = 0;
        $len = strlen($s);
        for ($i = 0; $i < $len; $i++) {
            $ch = $s[$i];
            if ($ch === '(') $stack++;
            if ($ch === ')') $stack--;
            if ($ch === '[') $stack2++;
            if ($ch === ']') $stack2--;
            if ($stack < 0 || $stack2 < 0) return false;
        }
        return $stack === 0 && $stack2 === 0;
    }

    /** ExpressionLanguage singleton con funciones matemáticas comunes registradas. */
    private static function getEL(): ExpressionLanguage
    {
        if (self::$el instanceof ExpressionLanguage) {
            return self::$el;
        }
        $el = new ExpressionLanguage();
        foreach (['sin', 'cos', 'tan', 'sqrt', 'log', 'exp', 'abs'] as $fn) {
            $el->addFunction(ExpressionFunction::fromPhp($fn));
        }
        self::$el = $el;
        return $el;
    }
}
