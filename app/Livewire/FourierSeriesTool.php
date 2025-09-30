<?php

namespace App\Livewire;

use App\Helper\MathHelper;
use Livewire\Component;
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
    public int  $terms_n        = 10; // Default value for the slider

    // Salidas evaluadas (listas para servir/descargar)
    public float $evaluated_a0 = 0.0;
    public array $evaluated_an = [];
    public array $evaluated_bn = [];

    // Estructura compacta para la vista y el frontend
    public array $seriesCoeffs = [
        'a0' => 0.0,
        'an' => [],
        'bn' => [],
        'period' => 0.0,
        'domainStart' => 0.0,
    ];

    /**
     * Acción principal: calcula coeficientes numéricamente.
     */
    public function calculate(): void
    {
        // Limpia buffers
        $this->evaluated_a0 = 0.0;
        $this->evaluated_an = [];
        $this->evaluated_bn = [];

        // Obtener los valores numéricos de los límites de forma segura
        $a_val = floatval(str_ireplace('pi', (string)\M_PI, $this->domainStart ?: '-pi'));
        $b_val = floatval(str_ireplace('pi', (string)\M_PI, $this->domainEnd ?: 'pi'));
        $T_val = $b_val - $a_val;

        // Crear la función PHP callable a partir de la definición del usuario
        $phpFunction = MathHelper::createPhpCallable($this->functionDefinition ?: 't');

        // --- CÁLCULO NUMÉRICO DE COEFICIENTES ---
        if ($T_val == 0) {
            Log::warning("Domain has zero length (T=0). Coefficients will be zero.");
        } else {
            // 1. Cálculo de a0 (como escalar)
            $integral_a0 = MathHelper::integrateNumerically($phpFunction, $a_val, $b_val);
            $this->evaluated_a0 = (1 / $T_val) * $integral_a0;
            Log::info("Fourier a0 (numeric): {$this->evaluated_a0}");

            // 2. Cálculo de an y bn (en un bucle)
            $N = 50; // Siempre calcular los 50 términos para la interactividad del frontend
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
        }

        // 3. Actualizar la estructura para la vista y el frontend
        $this->seriesCoeffs = [
            'a0' => $this->evaluated_a0,
            'an' => $this->evaluated_an,
            'bn' => $this->evaluated_bn,
            'period' => $T_val,
            'domainStart' => $a_val,
        ];

        // 4. Despachar evento para el frontend
        $this->dispatch(
            'fourier-series-updated',
            seriesCoeffs: $this->seriesCoeffs,
            functionDefinition: $this->functionDefinition ?: 't',
            renderOriginal: (bool)$this->renderOriginal,
            renderSeries: (bool)$this->renderSeries,
            terms_n: (int)$this->terms_n
        );
    }

    public function render()
    {
        return view('livewire.fourier-series-tool');
    }
}