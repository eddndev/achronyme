<?php

namespace App\Helper;

use Symfony\Component\ExpressionLanguage\ExpressionLanguage;
use Symfony\Component\ExpressionLanguage\ExpressionFunction;

class MathHelper
{
    /** @var ExpressionLanguage|null */
    private static ?ExpressionLanguage $el = null;

    /**
     * Normaliza una función “humana” a Wolfram Language.
     * Acepta: t, pi/Pi, sin|cos|tan|exp|log|abs|sqrt (con paréntesis).
     * Devuelve algo estilo: "t*Sin[3*t]" listo para Integrate[…].
     *
     * @throws \InvalidArgumentException
     */
    public static function normalizeFunctionForWolfram(string $expr): string
    {
        $expr = trim($expr);

        // Limpieza y normalizaciones básicas
        $expr = str_replace([' ', '‐', '–', '—'], '', $expr); // quita espacios/dashes raros
        $expr = preg_replace('/\bpi\b/i', 'Pi', $expr);       // pi -> Pi
        $expr = preg_replace('/\bsen\s*\(/i', 'sin(', $expr); // alias "sen(" -> "sin("

        // Verificación de paréntesis
        if (!self::isBalanced($expr)) {
            throw new \InvalidArgumentException('Paréntesis desbalanceados en la función.');
        }

        // Caracteres permitidos
        if (preg_match('/[^A-Za-z0-9_\+\-\*\/\^\.\(\)]/', $expr)) {
            throw new \InvalidArgumentException('Caracteres no permitidos en la función.');
        }

        // Identificadores alfabéticos (tokens)
        preg_match_all('/[A-Za-z_][A-Za-z0-9_]*/', $expr, $m);
        $idents = $m[0] ?? [];

        // Lista blanca de funciones y nombres
        $allowedFns = ['sin','cos','tan','exp','log','abs','sqrt'];
        $allowedNames = array_merge(['t','pi','Pi'], $allowedFns);

        foreach ($idents as $id) {
            $idLower = strtolower($id);
            if (!in_array($idLower, $allowedNames, true)) {
                throw new \InvalidArgumentException(
                    'Solo se permite la variable t y funciones conocidas (sin, cos, tan, exp, log, abs, sqrt).'
                );
            }
            // si es función, debe ir seguida de "("
            if (in_array($idLower, $allowedFns, true)) {
                $pos = stripos($expr, $id);
                if ($pos !== false) {
                    $next = substr($expr, $pos + strlen($id), 1);
                    if ($next !== '(') {
                        throw new \InvalidArgumentException("La función {$idLower} debe ir seguida de paréntesis: {$idLower}(...)");
                    }
                }
            }
        }

        // Normaliza potencias: ** -> ^
        $expr = preg_replace('/\*\*(?=\d|\()/','^',$expr);

        // Convierte funciones a Wolfram: sin( -> Sin[, etc.
        $mapFn = [
            '/\bsin\(/i'  => 'Sin[',
            '/\bcos\(/i'  => 'Cos[',
            '/\btan\(/i'  => 'Tan[',
            '/\bexp\(/i'  => 'Exp[',
            '/\blog\(/i'  => 'Log[',
            '/\babs\(/i'  => 'Abs[',
            '/\bsqrt\(/i' => 'Sqrt[',
        ];
        $expr = preg_replace(array_keys($mapFn), array_values($mapFn), $expr);

        // Cierra todas las llamadas de función reemplazando ) por ]
        $expr = str_replace(')', ']', $expr);

        // Validación final: solo t y Pi como nombres residuales (más los nombres de funciones ya transformadas con '[')
        preg_match_all('/[A-Za-z_][A-Za-z0-9_]*/', $expr, $m2);
        foreach ($m2[0] as $id2) {
            $id2lower = strtolower($id2);
            if (!in_array($id2lower, ['t','pi','sin','cos','tan','exp','log','abs','sqrt','sin[','cos[','tan[','exp[','log[','abs[','sqrt['], true)) {
                if ($id2 !== 'Pi') {
                    throw new \InvalidArgumentException('Solo se permite la variable t y la constante Pi.');
                }
            }
        }

        return $expr;
    }

    /**
     * Normaliza un límite para Wolfram: números, Pi, + - * / ^ y paréntesis.
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
     * Crea una función callable de PHP a partir de una expresión matemática en formato texto.
     * Utiliza ExpressionLanguage para una evaluación segura.
     *
     * @param string $expression La expresión matemática con 't' como variable.
     * @return callable Una función que acepta un float $t y devuelve un float.
     */
    public static function createPhpCallable(string $expression): callable
    {
        // Reemplaza el operador de potencia '^' por '**' que ExpressionLanguage entiende.
        $expression = str_replace('^', '**', $expression);

        return static function (float $t) use ($expression): float {
            $el = self::getEL(); // Reutiliza tu singleton de ExpressionLanguage
            try {
                // Evalúa la expresión pasando 't' y 'pi' como variables.
                return (float) $el->evaluate($expression, [
                    't' => $t,
                    'pi' => \M_PI,
                ]);
            } catch (\Throwable $e) {
                // En caso de un error de sintaxis en la función del usuario, devuelve 0.
                return 0.0;
            }
        };
    }

    /**
     * Calcula la integral definida de una función usando la regla de Simpson 1/3.
     *
     * @param callable $function La función a integrar (acepta un float, devuelve un float).
     * @param float $a Límite inferior de integración.
     * @param float $b Límite superior de integración.
     * @param int $steps El número de intervalos (debe ser par, se ajustará si es necesario).
     * @return float El valor aproximado de la integral.
     */
    public static function integrateNumerically(callable $function, float $a, float $b, int $steps = 1000): float
    {
        // La regla de Simpson requiere un número par de intervalos.
        if ($steps % 2 !== 0) {
            $steps++;
        }

        $h = ($b - $a) / $steps;
        $sum = $function($a) + $function($b);

        for ($i = 1; $i < $steps; $i++) {
            $x = $a + $i * $h;
            if ($i % 2 !== 0) {
                $sum += 4 * $function($x); // Términos impares
            } else {
                $sum += 2 * $function($x); // Términos pares
            }
        }

        return ($h / 3) * $sum;
    }

    /** ExpressionLanguage singleton con funciones registradas (incluye hiperbólicas). */
    private static function getEL(): ExpressionLanguage
    {
        if (self::$el instanceof ExpressionLanguage) {
            return self::$el;
        }
        $el = new ExpressionLanguage();

        // Nativas de PHP:
        foreach (['sin','cos','tan','sqrt','log','exp','abs','sinh','cosh','tanh'] as $fn) {
            $el->addFunction(ExpressionFunction::fromPhp($fn));
        }

        // Helpers para las que no existen en PHP: sech, csch, coth
        $el->addFunction(new ExpressionFunction(
            'sech',
            static fn ($v) => sprintf('(1/cosh(%s))', $v),
            static fn (array $vars, $v) => 1 / cosh($v)
        ));
        $el->addFunction(new ExpressionFunction(
            'csch',
            static fn ($v) => sprintf('(1/sinh(%s))', $v),
            static fn (array $vars, $v) => 1 / sinh($v)
        ));
        $el->addFunction(new ExpressionFunction(
            'coth',
            static fn ($v) => sprintf('(cosh(%s)/sinh(%s))', $v, $v),
            static fn (array $vars, $v) => cosh($v) / sinh($v)
        ));

        self::$el = $el;
        return self::$el;
    }

    /** Verificación ( ) y [ ] balanceados. */
    private static function balancedDelimiters(string $s): bool
    {
        $stackP = 0; // ()
        $stackB = 0; // []
        $len = strlen($s);
        for ($i = 0; $i < $len; $i++) {
            $ch = $s[$i];
            if ($ch === '(') $stackP++;
            if ($ch === ')') $stackP--;
            if ($ch === '[') $stackB++;
            if ($ch === ']') $stackB--;
            if ($stackP < 0 || $stackB < 0) return false;
        }
        return $stackP === 0 && $stackB === 0;
    }

    /** Verificación general de (), [], {} balanceados. */
    public static function isBalanced(string $s): bool
    {
        $stack = [];
        $pairs = [')' => '(', ']' => '[', '}' => '{'];
        $open  = array_values($pairs);

        $len = strlen($s);
        for ($i = 0; $i < $len; $i++) {
            $ch = $s[$i];
            if (in_array($ch, $open, true)) {
                $stack[] = $ch;
            } elseif (isset($pairs[$ch])) {
                if (empty($stack)) return false;
                $top = array_pop($stack);
                if ($top !== $pairs[$ch]) return false;
            }
        }
        return empty($stack);
    }
}
