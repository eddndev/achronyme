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

    // Resultados
    public array $results = [];
    public bool $isLoading = false;

    /**
     * Hook que se ejecuta cuando la propiedad $calculationMode cambia.
     */
    public function updatedCalculationMode($value)
    {
        if ($value === 'coefficients') {
            $this->renderOriginal = false;
        }
    }

    /**
     * Llama a la API de Wolfram para calcular los coeficientes.
     */
    public function calculate()
    {
        $this->isLoading = true;
        $this->results = [];
        $appId = config('services.wolfram.app_id');

        if (!$appId) {
            $this->results = ['error' => 'Wolfram App ID not configured.'];
            $this->isLoading = false;
            return;
        }

        $T = "({$this->domainEnd}) - ({$this->domainStart})";
        
        $queries = [
            'a0' => "1/($T) * integrate ({$this->functionDefinition}) from t={$this->domainStart} to {$this->domainEnd}",
            'an' => "2/($T) * integrate ({$this->functionDefinition}) * cos(2*pi*n*t/($T)) from t={$this->domainStart} to {$this->domainEnd}",
            'bn' => "2/($T) * integrate ({$this->functionDefinition}) * sin(2*pi*n*t/($T)) from t={$this->domainStart} to {$this->domainEnd}",
        ];

        foreach ($queries as $key => $query) {
            try {
                $response = Http::timeout(30)->get('https://api.wolframalpha.com/v2/query', [
                    'appid' => $appId,
                    'input' => $query,
                    'output' => 'json',
                    'format' => 'plaintext',
                ]);

                if ($response->failed()) {
                    throw new \Exception('API request failed for query: ' . $query);
                }

                $data = $response->json();
                $coefficient = 'Could not determine';

                if (($data['queryresult']['success'] ?? false) && isset($data['queryresult']['pods'])) {
                    foreach ($data['queryresult']['pods'] as $pod) {
                        if ($pod['title'] === 'Result' || str_contains($pod['title'], 'integral')) {
                            $coefficient = $pod['subpods'][0]['plaintext'];
                            break;
                        }
                    }
                }
                $this->results[$key] = $coefficient;

            } catch (\Exception $e) {
                Log::error("Wolfram API Error for $key: " . $e->getMessage(), ['query' => $query]);
                $this->results[$key] = 'Error calculating';
            }
        }

        $this->isLoading = false;
    }

    public function render()
    {
        return view('livewire.fourier-series-tool');
    }
}