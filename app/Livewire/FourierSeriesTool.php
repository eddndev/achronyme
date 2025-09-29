<?php

namespace App\Livewire;

use Livewire\Component;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;

class FourierSeriesTool extends Component
{
    // Estado de la UI
    public $calculationMode = 'calculate';

    // Inputs para el modo 'calculate'
    public $functionDefinition = 't';
    public $domainStart = '-pi';
    public $domainEnd = 'pi';

    // Inputs para el modo 'coefficients'
    public $coeff_a0 = '';
    public $coeff_an = '';
    public $coeff_bn = '';

    // Opciones de visualización
    public $renderOriginal = true;
    public $renderSeries = true;
    public $terms_n = 10;

    /**
     * Llama a la API de Wolfram para calcular los coeficientes de la serie de Fourier.
     */
    public function calculate()
    {
        $appId = config('services.wolfram.app_id');
        if (!$appId) {
            // Consider adding a user-facing error
            Log::error('Wolfram App ID not configured.');
            return;
        }

        $query = "fourier series of {$this->functionDefinition} from t={$this->domainStart} to {$this->domainEnd}";

        try {
            $response = Http::timeout(30)->get('https://api.wolframalpha.com/v2/query', [
                'appid' => $appId,
                'input' => $query,
                'output' => 'json',
                'format' => 'plaintext',
                'includepodid' => 'TrigonometricFourierSeries', // Specific pod for coefficients
            ]);

            if ($response->failed()) {
                throw new \Exception('API request failed.');
            }

            $data = $response->json();

            if (!($data['queryresult']['success'] ?? false) || !isset($data['queryresult']['pods'])) {
                throw new \Exception('API did not return a successful result or pods.');
            }

            $this->extractCoefficients($data['queryresult']['pods']);

            // Switch to coefficient mode to show results
            $this->calculationMode = 'coefficients';
            $this->renderOriginal = false;

        } catch (\Exception $e) {
            Log::error("Wolfram API Error: " . $e->getMessage(), ['query' => $query]);
            // Consider adding a user-facing error
        }
    }

    /**
     * Extrae los coeficientes de los pods de la API.
     */
    protected function extractCoefficients(array $pods)
    {
        $this->coeff_a0 = 'Not found';
        $this->coeff_an = 'Not found';
        $this->coeff_bn = 'Not found';

        foreach ($pods as $pod) {
            if ($pod['title'] === 'Trigonometric Fourier series') {
                foreach ($pod['subpods'] as $subpod) {
                    $text = $subpod['plaintext'];
                    if (str_starts_with($text, 'a_0')) {
                        $this->coeff_a0 = trim(explode('=', $text, 2)[1]);
                    } elseif (str_starts_with($text, 'a_n')) {
                        $this->coeff_an = trim(explode('=', $text, 2)[1]);
                    } elseif (str_starts_with($text, 'b_n')) {
                        $this->coeff_bn = trim(explode('=', $text, 2)[1]);
                    }
                }
                // Stop after finding the correct pod
                return;
            }
        }
    }

    public function render()
    {
        return view('livewire.fourier-series-tool');
    }
}