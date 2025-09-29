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
     * Convierte moutput (Wolfram Language) a una expresión evaluable por ExpressionLanguage.
     * - Pi->pi, ^->**, mapea Sin[...], Cos[...], Sinh[...], etc. a sin(...), cos(...), sinh(...).
     * - Inserta multiplicación implícita (2 n -> 2*n, n pi -> n*pi).
     */
    public static function wlToEL(string $s): string
    {
        // 1) Normalizaciones WL -> intermedio
        //    Convierte funciones WL a llamadas con '(' en vez de '['
        $s = preg_replace([
            '/\bSin\[/i','/\bCos\[/i','/\bTan\[/i',
            '/\bSinh\[/i','/\bCosh\[/i','/\bTanh\[/i',
            '/\bSech\[/i','/\bCsch\[/i','/\bCoth\[/i',
            '/\bExp\[/i','/\bLog\[/i','/\bAbs\[/i','/\bSqrt\[/i',
        ], [
            'sin(','cos(','tan(',
            'sinh(','cosh(','tanh(',
            'sech(','csch(','coth(',
            'exp(','log(','abs(','sqrt(',
        ], $s);

        // Corchetes -> paréntesis
        $s = str_replace(['[',']'], ['(',')'], $s);
        // Pi -> pi, ^ -> ** (operador de potencia)
        $s = str_replace(['π','Pi','^'], ['pi','pi','**'], $s);
        // Quitar espacios (ya no nos afecta porque tokenizaremos)
        $s = preg_replace('/\s+/', '', $s);

        $s = preg_replace('/\bE\s*\*\*\s*\(([^)]+)\)/', 'exp($1)', $s);
        // 2) E**algo_simple (pi, n, número o paréntesis simple)
        $s = preg_replace('/\bE\s*\*\*\s*([A-Za-z0-9_\.]+|\([^()]*\))/', 'exp($1)', $s);
        // 3) E suelta -> exp(1)
        $s = preg_replace('/\bE\b/', 'exp(1)', $s);

        // 2) Tokeniza con reglas que reconocen funciones, n, pi, números, paréntesis y operadores
        $tokens = self::tokenizeEL($s);

        // 3) Reconstruye inyectando multiplicación implícita cuando corresponda
        $out = self::rebuildWithImplicitMultiplication($tokens);

        return $out;
    }

    /** Evalúa una expresión EL que NO depende de n (ej. 0, 1/pi, exp(pi)/(2*pi)). */
    public static function evalScalarEL(string $exprEL): float
    {
        $el = self::getEL();
        $v = $el->evaluate($exprEL, ['pi' => \M_PI]); // si usaste exp(1) ya no necesitas 'e'
        $eps = 1e-12;
        if (is_numeric($v)) {
            if (abs($v) < $eps) $v = 0.0;
            if (abs($v - round($v)) < $eps) $v = (float) round($v);
        }
        return (float) $v;
    }

    /** Replica un escalar en un arreglo [1..N] => valor. */
    public static function replicateScalar(float $value, int $N): array
    {
        $out = [];
        for ($n = 1; $n <= $N; $n++) $out[$n] = $value;
        return $out;
    }


    /** Tokeniza una cadena EL en números, identificadores (funciones, n, pi), paréntesis y operadores. */
    private static function tokenizeEL(string $s): array
    {
        $funcs = ['sinh','cosh','tanh','sech','csch','coth','sin','cos','tan','sqrt','exp','log','abs'];
        // ordenar por longitud desc para hacer matching codicioso
        usort($funcs, fn($a,$b) => strlen($b) <=> strlen($a));

        $tokens = [];
        $i = 0; $len = strlen($s);

        while ($i < $len) {
            // ** operador
            if ($i+1 < $len && $s[$i] === '*' && $s[$i+1] === '*') { $tokens[]='**'; $i+=2; continue; }

            $ch = $s[$i];

            // números (enteros o decimales)
            if (preg_match('/\G\d+(\.\d+)?/A', $s, $m, 0, $i)) {
                $tokens[] = $m[0];
                $i += strlen($m[0]);
                continue;
            }

            // paréntesis u operadores simples
            if (strpos('()+-*/', $ch) !== false) { $tokens[] = $ch; $i++; continue; }

            // funciones (codicioso)
            $matched = false;
            foreach ($funcs as $fn) {
                $L = strlen($fn);
                if ($i + $L <= $len && strncasecmp($s, $fn, $L) === 0) {
                    $tokens[] = strtolower($fn);
                    $i += $L;
                    $matched = true;
                    break;
                }
            }
            if ($matched) continue;

            // constantes/variables
            if ($i+1 < $len && strncasecmp($s, 'pi', 2) === 0) { $tokens[]='pi'; $i+=2; continue; }
            if ($ch === 'n' || $ch === 'N') { $tokens[]='n'; $i++; continue; }

            // run de letras desconocidas -> intenta descomponer en (n|pi|funciones)
            if (preg_match('/\G[A-Za-z_]+/A', $s, $m, 0, $i)) {
                $run = $m[0]; $rlen = strlen($run); $j = 0;
                while ($j < $rlen) {
                    $subMatched = false;
                    foreach ($funcs as $fn) {
                        $L = strlen($fn);
                        if ($j + $L <= $rlen && strncasecmp(substr($run, $j, $L), $fn, $L) === 0) {
                            $tokens[] = strtolower($fn);
                            $j += $L;
                            $subMatched = true;
                            break;
                        }
                    }
                    if ($subMatched) continue;

                    if ($j + 2 <= $rlen && strncasecmp(substr($run, $j, 2), 'pi', 2) === 0) {
                        $tokens[] = 'pi'; $j += 2; continue;
                    }
                    if (strtolower($run[$j]) === 'n') { $tokens[]='n'; $j++; continue; }

                    // si llegamos aquí, hay un símbolo que no reconocemos
                    throw new \InvalidArgumentException("Token no reconocido en '{$run}' cerca de '".substr($run, $j)."'");
                }
                $i += $rlen;
                continue;
            }

            throw new \InvalidArgumentException("Carácter no reconocido: '{$ch}'");
        }

        return $tokens;
    }

    /** Inserta '*' entre tokens donde hay multiplicación implícita. */
    private static function rebuildWithImplicitMultiplication(array $tokens): string
    {
        $funcs = ['sinh','cosh','tanh','sech','csch','coth','sin','cos','tan','sqrt','exp','log','abs'];

        $isNum   = fn($t) => (bool) preg_match('/^\d+(\.\d+)?$/', $t);
        $isIdent = fn($t) => (bool) preg_match('/^[A-Za-z_][A-Za-z_0-9]*$/', $t);
        $isFunc  = fn($t) => $isIdent($t) && in_array(strtolower($t), $funcs, true);
        $isSym   = fn($t) => $t === 'n' || $t === 'pi';

        $out = '';
        $prev = null;

        foreach ($tokens as $tok) {
            if ($prev !== null) {
                $prevIsNum   = $isNum($prev);
                $prevIsSym   = $isSym($prev);
                $prevIsFunc  = $isFunc($prev);
                $prevIsClose = ($prev === ')');

                $curIsNum    = $isNum($tok);
                $curIsSym    = $isSym($tok);
                $curIsFunc   = $isFunc($tok);
                $curIsOpen   = ($tok === '(');

                $mustMultiply =
                    // (num|n|pi|')') seguido de (num|n|pi|'('|func)
                    (($prevIsNum || $prevIsSym || $prevIsClose) && ($curIsNum || $curIsSym || $curIsOpen || $curIsFunc))
                    ||
                    // función seguida de símbolo/num/otra función (ej. sin n, cos 2x, etc.)
                    ($prevIsFunc && ($curIsNum || $curIsSym || $curIsFunc))
                    ||
                    // símbolo seguido de apertura de paréntesis: n(…)
                    ($prevIsSym && $curIsOpen)
                    ||
                    // cierre de paréntesis seguido de función: )sin(
                    ($prevIsClose && $curIsFunc);

                // pero jamás entre función y su '(' inmediato: sin (
                if ($prevIsFunc && $curIsOpen) {
                    $mustMultiply = false;
                }

                if ($mustMultiply) {
                    $out .= '*';
                }
            }

            $out .= $tok;
            $prev = $tok;
        }

        return $out;
    }



    /**
     * Evalúa una expresión EL para n = 1..N y devuelve [n => valor].
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
