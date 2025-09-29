<!-- 3 column wrapper -->
<div class="mx-auto w-full max-w-7xl grow lg:flex xl:px-2" x-data="{ calculationMode: 'calculate' }">
    <!-- Left sidebar & main wrapper -->
    <div class="flex-1 xl:flex">
        <div class="border-b border-gray-200 p-6 sm:px-6 lg:pl-8 xl:w-96 xl:shrink-0 xl:border-r xl:border-b-0 xl:pl-6 dark:border-white/10">
            <!-- Left sidebar content -->
            <div class="space-y-6">
                @php
                $calcOptions = [
                    [
                        'value' => 'calculate',
                        'title' => 'Calcular coeficientes',
                        'description' => 'Define una función f(t) y su dominio para calcular a₀, aₙ y bₙ.'
                    ],
                    [
                        'value' => 'coefficients',
                        'title' => 'Ingresar coeficientes',
                        'description' => 'Introduce manualmente los valores de los coeficientes de la serie.'
                    ]
                ];
                @endphp
                <div @change="calculationMode = $event.target.value">
                    <x-app-ui.radio-list
                        legend="Modo de cálculo"
                        name="calculation_mode"
                        :options="$calcOptions"
                        checkedValue="calculate"
                    />
                </div>

                <div x-show="calculationMode === 'calculate'" x-transition class="space-y-4">
                    <h3 class="font-medium text-slate-900 dark:text-white">Función y Dominio</h3>
                    <x-app-ui.function-domain
                        wire:function.defer="functionDefinition"
                        wire:domainStart.defer="domainStart"
                        wire:domainEnd.defer="domainEnd"
                        functionName="function_definition"
                        functionPlaceholder="Ej: t"
                        domainStartName="domain_start"
                        domainStartPlaceholder="-pi"
                        domainEndName="domain_end"
                        domainEndPlaceholder="pi"
                    />
                </div>

                <div x-show="calculationMode === 'coefficients'" x-transition class="space-y-4">
                    <h3 class="font-medium text-slate-900 dark:text-white">Coeficientes de Fourier</h3>
                    <x-app-ui.input-text label="Coeficiente a₀" name="coeff_a0" placeholder="Ej: 1/2" wire:model.defer="coeff_a0" />
                    <x-app-ui.input-text label="Coeficiente aₙ" name="coeff_an" placeholder="Ej: (2/(n*pi))*sin(n*pi/2)" wire:model.defer="coeff_an" />
                    <x-app-ui.input-text label="Coeficiente bₙ" name="coeff_bn" placeholder="Ej: 0" wire:model.defer="coeff_bn" />
                </div>
            </div>
        </div>

        <div class="px-4 py-6 sm:px-6 lg:pl-8 xl:flex-1 xl:pl-6">
            <!-- Main content -->
            @if (isset($debugOutput) && $debugOutput)
            <div class="mt-6">
                <h3 class="font-medium text-slate-900 dark:text-white">Resultados de los Coeficientes</h3>
                <div class="mt-2 space-y-6">
                    @foreach($debugOutput as $key => $value)
                        <div class="flex items-start">
                            <span class="font-bold w-12 text-right pr-4 pt-2">{{ $key }} =</span>
                            <div class="p-4 bg-slate-100 dark:bg-slate-800 rounded-md overflow-x-auto flex-1">
                                {!! $value !!}
                            </div>
                        </div>
                    @endforeach
                </div>

                <h3 class="mt-8 font-medium text-slate-900 dark:text-white">Raw MathML Output</h3>
                <pre class="mt-2 text-xs bg-slate-100 dark:bg-slate-800 p-4 rounded-md overflow-x-auto">@php print_r(collect($debugOutput)->map(fn ($item) => htmlspecialchars($item))->all()) @endphp</pre>


            </div>
            @endif
        </div>
    </div>

    <div class="shrink-0 border-t border-gray-200 p-6 sm:px-6 lg:w-96 lg:border-t-0 lg:border-l lg:pr-8 xl:pr-6 dark:border-white/10">
        <!-- Right sidebar content -->
        <div class="space-y-6">
            <div x-show="calculationMode === 'calculate'" x-transition>
                @php
                $renderOriginalOption = [
                    [
                        'name' => 'render_options[original]',
                        'value' => 'original',
                        'title' => 'Renderizar función original',
                        'description' => 'Muestra la gráfica de f(t).'
                    ]
                ];
                @endphp
                <x-app-ui.checkbox-list
                    legend="Opciones de visualización"
                    :options="$renderOriginalOption"
                    :checkedValues="['original']"
                    wire:model.defer="renderOriginal"
                />
            </div>

            @php
            $renderSeriesOption = [
                [
                    'name' => 'render_options[series]',
                    'value' => 'series',
                    'title' => 'Renderizar serie de Fourier',
                    'description' => 'Muestra la gráfica de la aproximación.'
                ]
            ];
            @endphp
            <x-app-ui.checkbox-list
                :options="$renderSeriesOption"
                :checkedValues="['series']"
                wire:model.defer="renderSeries"
            />

            <x-app-ui.slider label="Número de términos (N)" name="num_terms" min="1" max="50" step="1" value="10" wire:model.defer="terms_n" />

            <x-primary-button type="button" class="w-full flex justify-center items-center" wire:click="calculate" wire:loading.attr="disabled">
                <span>Calcular y Graficar</span>
                <svg wire:loading.delay wire:target="calculate" class="animate-spin ml-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
            </x-primary-button>
        </div>
    </div>
</div>