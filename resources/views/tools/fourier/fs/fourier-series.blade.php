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
                <div x-show="calculationMode === 'calculate'" x-transition>
                    <fieldset>
                        <legend class="text-sm font-medium leading-6 text-slate-900 dark:text-white">Opciones de visualización</legend>
                        <div class="mt-2 space-y-4">
                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_original" value="original" x-model="renderOriginal" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_original" class="font-medium text-slate-900 dark:text-white">Renderizar función original</label>
                                    <p class="text-slate-500 dark:text-slate-400">Muestra la gráfica de f(t).</p>
                                </div>
                            </div>
                        </div>
                    </fieldset>
                </div>

                <fieldset>
                    <div class="mt-6 space-y-4">
                        <div class="relative flex items-start">
                            <div class="flex h-6 items-center">
                                <x-app-ui.checkbox id="render_series" value="series" x-model="renderSeries" />
                            </div>
                            <div class="ml-3 text-sm leading-6">
                                <label for="render_series" class="font-medium text-slate-900 dark:text-white">Renderizar serie de Fourier</label>
                                <p class="text-slate-500 dark:text-slate-400">Muestra la gráfica de la aproximación.</p>
                            </div>
                        </div>
                    </div>
                </fieldset>

                <x-app-ui.slider label="Número de términos (N)" name="num_terms" min="1" max="50" step="1" value="10" x-model="terms_n" />
            </div>
            
        </div>

        <div class="min-w-0 px-4 py-6 sm:px-6 lg:pl-8 xl:flex-1 xl:pl-6">
            <!-- Main content -->
            <div class="mb-8">
                <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                    <div class="relative w-full" style="height: 500px; max-width: 100%;" id="chartContainer">
                        <canvas id="fourierChart"></canvas>
                    </div>
                </div>
            </div>

        </div>
    </div>

    <div class="shrink-0 border-t border-gray-200 p-6 sm:px-6 lg:w-96 lg:border-t-0 lg:border-l lg:pr-8 xl:pr-6 dark:border-white/10">
        <!-- Right sidebar content -->
        <div class="space-y-6">
            

            <div x-show="calculationMode === 'calculate'" x-transition class="space-y-4">
                <h3 class="font-medium text-slate-900 dark:text-white">Funciones y Dominios</h3>

                <template x-for="(func, index) in functions" :key="func.id">
                    <div class="relative">
                        <x-app-ui.function-domain
                            x-model-function="func.definition"
                            x-model-domain-start="func.domainStart"
                            x-model-domain-end="func.domainEnd"
                            function-placeholder="Ej: t"
                            domain-start-placeholder="-pi"
                            domain-end-placeholder="pi"
                            :function-error-model="'func.definitionError'"
                            :domain-start-error-model="'func.domainStartError'"
                            :domain-end-error-model="'func.domainEndError'"
                            ::is-domain-start-disabled="index > 0"
                            ::index="index"
                        />
                        <template x-if="functions.length > 1">
                            <div class="absolute top-1 right-1">
                                <x-app-ui.circular-button
                                    icon="trash"
                                    @click.prevent="removeFunction(func.id)"
                                    class="!bg-red-500 hover:!bg-red-600 focus:!ring-red-500"
                                    title="Eliminar función"
                                />
                            </div>
                        </template>
                    </div>
                </template>

                <div class="pt-2">
                    <button type="button" @click.prevent="addFunction()" class="inline-flex items-center gap-x-1.5 rounded-md bg-white px-2.5 py-1.5 text-sm font-semibold text-slate-900 shadow-sm ring-1 ring-inset ring-slate-300 hover:bg-slate-50 dark:bg-white/10 dark:text-white dark:ring-slate-700 dark:hover:bg-white/20">
                        <svg class="size-5 -ml-0.5" fill="currentColor"><use href="#icon-plus"></use></svg>
                        Añadir función
                    </button>
                </div>
            </div>

            <div x-show="calculationMode === 'coefficients'" x-transition class="space-y-4">
                <h3 class="font-medium text-slate-900 dark:text-white">Coeficientes de Fourier</h3>
                <x-app-ui.input-text label="Coeficiente a₀" name="coeff_a0" placeholder="Ej: 1/2" x-model="coeff_a0_str" />
                <x-app-ui.input-text label="Coeficiente aₙ" name="coeff_an" placeholder="Ej: (2/(n*pi))*sin(n*pi/2)" x-model="coeff_an_str" />
                <x-app-ui.input-text label="Coeficiente bₙ" name="coeff_bn" placeholder="Ej: 0" x-model="coeff_bn_str" />
            </div>
            <div class="flex">
                <x-app-ui.button class="h-12" type="button" @click="calculateAndRedraw()" loading-text="Calculando...">
                    Calcular
                </x-app-ui.button>
            </div>
        </div>
    </div>
</div>
