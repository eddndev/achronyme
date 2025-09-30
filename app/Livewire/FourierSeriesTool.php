<?php

namespace App\Livewire;

use App\Helper\MathHelper;
use Livewire\Component;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
class FourierSeriesTool extends Component
{
    // Estado de la UI
    public string $calculationMode = 'calculate';

    // Inputs para el modo 'calculate'
    public string $functionDefinition = 't';
    public string $domainStart = '-pi';
    public string $domainEnd   = 'pi';

    // Inputs para el modo 'coefficients' (si más tarde quieres soportar entrada manual)
    public string $coeff_a0 = '';
    public string $coeff_an = '';
    public string $coeff_bn = '';

    // Opciones de visualización
    public bool $renderOriginal = true;
    public bool $renderSeries   = true;
    public int  $terms_n        = 50; // slider en la UI

    // Salidas de depuración (MathML en crudo y moutput simbólico)
    public array $debugOutput          = []; // key => mathml

    // Salidas evaluadas (listas para servir/descargar)
    public float $evaluated_a0 = 0.0;  // [n=>valor]
    public array $evaluated_an = [];  // [n=>valor]
    public array $evaluated_bn = [];  // [n=>valor]

    // Estructura compacta para la vista
    public array $seriesCoeffs = [
        'a0' => [],
        'an' => [],
        'bn' => [],
    ];

    /**
     * Acción principal: calcula coeficientes (símbolos + evaluación n=1..N).
     */
    public function calculate(): void
    {
        $appId = config('services.wolfram.app_id');
        if (!$appId) {
            Log::error('Wolfram App ID not configured.');
            return;
        }

        // Limpia buffers
        $this->debugOutput = [];
        $this->evaluated_a0 = 0.0;
        $this->evaluated_an = $this->evaluated_bn = [];
        $this->seriesCoeffs = ['a0' => 0.0, 'an' => [], 'bn' => []];

        // Obtener los valores numéricos de los límites de forma segura
        $a_val = floatval(str_ireplace('pi', (string)\M_PI, $this->domainStart ?: '-pi'));
        $b_val = floatval(str_ireplace('pi', (string)\M_PI, $this->domainEnd ?: 'pi'));
        $T_val = $b_val - $a_val;

        // Crear la función PHP callable a partir de la definición del usuario
        $phpFunction = MathHelper::createPhpCallable($this->functionDefinition ?: 't');

        // 1) Normaliza entradas de la UI a sintaxis Wolfram segura (para visualización)
        try {
            $f = MathHelper::normalizeFunctionForWolfram($this->functionDefinition ?: 't');
            $a = MathHelper::normalizeLimitForWolfram($this->domainStart   ?: '-Pi');
            $b = MathHelper::normalizeLimitForWolfram($this->domainEnd     ?: 'Pi');
        } catch (\Throwable $e) {
            Log::error('Input normalization error: '.$e->getMessage(), [
                'function' => $this->functionDefinition,
                'a' => $this->domainStart,
                'b' => $this->domainEnd,
            ]);
            return;
        }

        // 2) Construye queries para intervalo [a,b] para Wolfram (SOLO VISUALIZACIÓN)
        $T = "({$b})-({$a})";
        $queries = [
            'a0' => "(1/({$T})) * Integrate[({$f}), {t, {$a}, {$b}}]",
            'an' => "(2/({$T})) * Integrate[({$f}) * Cos[2*Pi*n*t/({$T})], {t, {$a}, {$b}}]",
            'bn' => "(2/({$T})) * Integrate[({$f}) * Sin[2*Pi*n*t/({$T})], {t, {$a}, {$b}}]",
        ];

        foreach ($queries as $key => $query) {
            // --- BLOQUE 1: Llamada a Wolfram para obtener MathML ---
            try {
                $params = [
                    'appid'  => $appId,
                    'input'  => $query,
                    'output' => 'json',
                    'format' => 'mathml', // Ya no pedimos moutput
                ];

                $response = Http::timeout(30)->get('https://api.wolframalpha.com/v2/query', $params);
                if ($response->failed()) {
                    throw new \Exception("API request failed for query: {$key}");
                }

                $data = $response->json();
                if (!(bool)($data['queryresult']['success'] ?? false) || empty($data['queryresult']['pods'])) {
                    Log::warning("Wolfram query for '{$key}' not successful or no pods.", ['query' => $query]);
                    continue;
                }

                $pods = $data['queryresult']['pods'];
                $formats = $this->extractFormats($pods);
                $this->debugOutput[$key] = $formats['mathml'];

            } catch (\Throwable $e) {
                Log::error("Wolfram API error for {$key}: ".$e->getMessage(), ['query' => $query]);
            }
        }

        // --- BLOQUE 2: CÁLCULO NUMÉRICO DE COEFICIENTES ---
        if ($T_val == 0) {
            Log::warning("Domain has zero length (T=0). Coefficients will be zero.");
            $this->evaluated_a0 = 0.0;
            $this->evaluated_an = [];
            $this->evaluated_bn = [];
        } else {
            // 1. Cálculo de a0 (como escalar)
            $integral_a0 = MathHelper::integrateNumerically($phpFunction, $a_val, $b_val);
            $this->evaluated_a0 = (1 / $T_val) * $integral_a0;
            Log::info("Fourier a0 (numeric): {$this->evaluated_a0}");

            // 2. Cálculo de an y bn (en un bucle)
            $N = max(1, min(50, (int)$this->terms_n));
            $this->evaluated_an = [];
            $this->evaluated_bn = [];

            for ($n = 1; $n <= $N; $n++) {
                // Integrando para an
                $integrand_an = static fn($t) => $phpFunction($t) * cos(2 * M_PI * $n * $t / $T_val);
                $integral_an = MathHelper::integrateNumerically($integrand_an, $a_val, $b_val);
                $this->evaluated_an[$n] = (2 / $T_val) * $integral_an;

                // Integrando para bn
                $integrand_bn = static fn($t) => $phpFunction($t) * sin(2 * M_PI * $n * $t / $T_val);
                $integral_bn = MathHelper::integrateNumerically($integrand_bn, $a_val, $b_val);
                $this->evaluated_bn[$n] = (2 / $T_val) * $integral_bn;
            }
            Log::info("Fourier an n=1..{$N}", $this->evaluated_an);
            Log::info("Fourier bn n=1..{$N}", $this->evaluated_bn);
        }

        // 3. Actualizar la estructura para la vista
        $this->seriesCoeffs = [
            'a0' => $this->evaluated_a0, // Ahora es un escalar
            'an' => $this->evaluated_an,
            'bn' => $this->evaluated_bn,
        ];

        // 4. Despachar evento para el frontend
        $this->dispatch(
            'fourier-series-updated',
            seriesCoeffs: $this->seriesCoeffs,
            domainStart: $a_val,
            domainEnd: $b_val,
            functionDefinition: $this->functionDefinition ?: 't',
            renderOriginal: (bool)$this->renderOriginal,
            renderSeries: (bool)$this->renderSeries,
        );

        // Por si quieres forzar a que “calcule y grafique”
        $this->renderOriginal = (bool)$this->renderOriginal;
        $this->renderSeries   = (bool)$this->renderSeries;
    }

    /**
     * Extrae los formatos mathml y moutput de una lista de pods.
     * (Esta es tu implementación “buena” que ya funcionaba; la dejo igual)
     */
    protected function extractFormats(array $pods): array
    {
        $formats = [
            'mathml'  => '<math><mtext>Not found</mtext></math>',
            'moutput' => 'Not found',
        ];

        // Prioriza pods con valor simbólico claro
        foreach ($pods as $pod) {
            if (in_array(($pod['id'] ?? ''), ['DefiniteIntegral', 'Result'], true)) {
                $sub = $pod['subpods'][0] ?? null;
                if ($sub) {
                    if (!empty($sub['mathml']))  $formats['mathml']  = $sub['mathml'];
                    if (!empty($sub['moutput'])) $formats['moutput'] = $sub['moutput'];
                    return $formats;
                }
            }
        }

        // Fallback al primer pod con contenido
        $first = $pods[0]['subpods'][0] ?? null;
        if ($first) {
            if (!empty($first['mathml']))  $formats['mathml']  = $first['mathml'];
            if (!empty($first['moutput'])) $formats['moutput'] = $first['moutput'];
        }

        return $formats;
    }

    public function render()
    {
        return view('livewire.fourier-series-tool');
    }
}