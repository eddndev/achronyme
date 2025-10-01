<div 
    class="mx-auto w-full max-w-7xl grow lg:flex xl:px-2"
    x-data="fourierState()"
    x-init="init()"
>
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
                        'description' => 'Introduce manually los valores de los coeficientes de la serie.'
                    ]
                ];
                @endphp
                <div>
                    <x-app-ui.radio-list
                        legend="Modo de cálculo"
                        name="calculation_mode"
                        :options="$calcOptions"
                        checkedValue="calculate"
                        x-model="calculationMode"
                    />
                </div>

                <div x-show="calculationMode === 'calculate'" x-transition class="space-y-4">
                    <h3 class="font-medium text-slate-900 dark:text-white">Función y Dominio</h3>
                    <x-app-ui.function-domain
                        xModelFunction="functionDefinition"
                        xModelDomainStart="domainStart"
                        xModelDomainEnd="domainEnd"
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
                    <x-app-ui.input-text label="Coeficiente" name="coeff_a0" placeholder="Ej: 1/2" x-model="coeff_a0" />
                    <x-app-ui.input-text label="Coeficiente" name="coeff_an" placeholder="Ej: (2/(n*pi))*sin(n*pi/2)" x-model="coeff_an" />
                    <x-app-ui.input-text label="Coeficiente" name="coeff_bn" placeholder="Ej: 0" x-model="coeff_bn" />
                </div>
            </div>
        </div>

        <div class="px-4 py-6 sm:px-6 lg:pl-8 xl:flex-1 xl:pl-6">
            <!-- Main content -->
            <div class="mb-8">
                <h3 class="font-medium text-slate-900 dark:text-white">Visualización de la Serie</h3>
                <div class="mt-4 bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                    <canvas id="fourierChart" class="w-full"></canvas>
                </div>
            </div>


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
                    x-model="renderOriginal"
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
                x-model="renderSeries"
            />

            <x-app-ui.slider label="Número de términos (N)" name="num_terms" min="1" max="50" step="1" value="10" x-model="terms_n" />

            <x-app-ui.button type="button" @click="calculateAndRedraw()" is-loading="isLoading" loading-text="Calculando...">
                Calcular
            </x-app-ui.button>
        </div>
    </div>
</div>
