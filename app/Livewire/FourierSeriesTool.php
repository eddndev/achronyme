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
    public array $moutputCoefficients  = []; // key => moutput

    // Salidas evaluadas (listas para servir/descargar)
    public array $evaluated_a0 = [];  // [n=>valor]
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
        $this->debugOutput         = [];
        $this->moutputCoefficients = [];
        $this->evaluated_a0 = $this->evaluated_an = $this->evaluated_bn = [];
        $this->seriesCoeffs = ['a0'=>[], 'an'=>[], 'bn'=>[]];

        // 1) Normaliza entradas de la UI a sintaxis Wolfram segura
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

        // 2) Construye queries para intervalo [a,b]
        $T = "({$b})-({$a})";
        $queries = [
            'a0' => "(1/({$T})) * Integrate[({$f}), {t, {$a}, {$b}}]",
            'an' => "(2/({$T})) * Integrate[({$f}) * Cos[2*Pi*n*t/({$T})], {t, {$a}, {$b}}]",
            'bn' => "(2/({$T})) * Integrate[({$f}) * Sin[2*Pi*n*t/({$T})], {t, {$a}, {$b}}]",
        ];

        $N = max(1, min(50, (int) $this->terms_n));
        $zero = array_fill_keys(range(1, $N), 0.0);
        $this->evaluated_a0 = $zero;
        $this->evaluated_an = $zero;
        $this->evaluated_bn = $zero;

        $N = max(1, min(50, (int)$this->terms_n)); // límite seguro

        foreach ($queries as $key => $query) {
            $pods = null;
            $best = null;

            // --- BLOQUE 1: Llamada a Wolfram + extracción de formatos ---
            try {
                $params = [
                    'appid'  => $appId,
                    'input'  => $query,
                    'output' => 'json',
                    'format' => 'mathml,moutput', // nos aseguramos de pedir moutput
                ];

                $response = Http::timeout(30)->get('https://api.wolframalpha.com/v2/query', $params);
                if ($response->failed()) {
                    throw new \Exception("API request failed for query: {$key}");
                }

                $data = $response->json();
                Log::debug("Full Wolfram API response for '{$key}'", $data);

                if (!(bool)($data['queryresult']['success'] ?? false) || empty($data['queryresult']['pods'])) {
                    Log::warning("Wolfram query for '{$key}' not successful or no pods.", [
                        'query' => $query,
                        'response' => $data,
                    ]);
                    continue;
                }

                $pods = $data['queryresult']['pods'];

                // Mantén tu extractor tal cual (prefiere DefiniteIntegral / Result)
                $formats = $this->extractFormats($pods);
                $this->debugOutput[$key]         = $formats['mathml'];
                $this->moutputCoefficients[$key] = $formats['moutput'];

                // Elige el mejor moutput evaluable (Result → DefiniteIntegral → …)
                $best = $this->pickBestMoutput($pods);
                if ($best === null) {
                    Log::warning("No evaluable moutput for {$key}");
                    continue;
                }
                Log::info("Fourier {$key} expr (moutput): {$best}");
            } catch (\Throwable $e) {
                Log::error("Wolfram API error for {$key}: ".$e->getMessage(), ['query' => $query]);
                continue;
            }

            // --- BLOQUE 2: Conversión a EL + evaluación n=1..N ---
            try {
                $exprEL = MathHelper::wlToEL($best);
                Log::info("Fourier {$key} expr (EL): {$exprEL}");

                if (preg_match('/\bn\b/i', $exprEL)) {
                    // depende de n -> evaluar para 1..N
                    $vals = MathHelper::evalForNsEL($exprEL, $N);
                } else {
                    // escalar (incluye '0', '0.', 'exp(pi)/(2*pi)', etc.) -> evalúa 1 vez y replica
                    $scalar = MathHelper::evalScalarEL($exprEL);
                    $vals   = MathHelper::replicateScalar($scalar, $N);
                }

                if ($key === 'a0') $this->evaluated_a0 = $vals;
                if ($key === 'an') $this->evaluated_an = $vals;
                if ($key === 'bn') $this->evaluated_bn = $vals;

                Log::info("Fourier {$key} n=1..{$N}", $vals);
            } catch (\Throwable $e) {
                Log::error("Evaluation error for {$key}: ".$e->getMessage(), ['best' => $best]);
            }
        }

        // 3) Deja armado lo que la UI podría necesitar directamente
        $this->seriesCoeffs = [
            'a0' => $this->evaluated_a0,
            'an' => $this->evaluated_an,
            'bn' => $this->evaluated_bn,
        ];

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

    /**
     * Prioriza un moutput “evaluable” del conjunto de pods.
     * Orden: Result → DefiniteIntegral → ExpandedForm → AlternateForm → cualquier otro.
     */
    protected function pickBestMoutput(array $pods): ?string
    {
        $order = ['Result', 'DefiniteIntegral', 'ExpandedForm', 'AlternateForm'];
        foreach ($order as $want) {
            foreach ($pods as $pod) {
                if (($pod['id'] ?? '') === $want) {
                    foreach (($pod['subpods'] ?? []) as $sub) {
                        if (array_key_exists('moutput', $sub)) {
                            $val = (string)$sub['moutput'];      // "0" OK
                            if ($val !== '') return trim($val);
                        }
                    }
                }
            }
        }
        // Fallback: primer moutput disponible
        foreach ($pods as $pod) {
            foreach (($pod['subpods'] ?? []) as $sub) {
                if (!empty($sub['moutput'])) {
                    return trim($sub['moutput']);
                }
            }
        }
        return null;
    }

    public function render()
    {
        return view('livewire.fourier-series-tool');
    }
}