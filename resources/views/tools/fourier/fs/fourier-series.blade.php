<div x-data="fourierState()" x-init="init()">
    <x-three-column-tool>
        {{-- Left Sidebar: Controls --}}
        <x-slot:leftSidebar>
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
                    <div class="space-y-4">
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
        </x-slot>

        {{-- Center: Chart --}}
        <div class="mb-8">
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <div class="relative w-full" style="height: 500px; max-width: 100%;" id="chartContainer">
                    <canvas id="fourierChart"></canvas>
                </div>
            </div>
        </div>

        {{-- Right Sidebar: Inputs --}}
        <x-slot:rightSidebar>
            <div class="space-y-6">
                <div x-show="calculationMode === 'calculate'" x-transition class="space-y-4">
                    <h3 class="font-medium text-slate-900 dark:text-white">Funciones y Dominios</h3>

                    <template x-for="(func, index) in functions" :key="func.id">
                        <div class="relative">
                            <template x-if="functions.length > 1">
                                <div class="absolute -top-2 -right-2 z-10">
                                    <x-app-ui.danger-circular-button
                                        icon="trash"
                                        @click.prevent="removeFunction(func.id)"
                                        title="Eliminar función"
                                    />
                                </div>
                            </template>
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
                        </div>
                    </template>

                    <div class="pt-2">
                        <x-app-ui.secondary-button type="button" @click.prevent="addFunction()">
                            <div class="flex space-x-2 items-center">
                                <svg class="size-5 -ml-0.5" fill="currentColor"><use href="#icon-plus"></use></svg>
                                <div>Agregar función</div>
                            </div>
                        </x-app-ui.secondary-button>
                    </div>
                </div>

                <div x-show="calculationMode === 'coefficients'" x-transition class="space-y-4">
                    <h3 class="font-medium text-slate-900 dark:text-white">Coeficientes de Fourier</h3>
                    <x-app-ui.input-text
                        label="Coeficiente a₀"
                        name="coeff_a0"
                        placeholder="Ej: 1/2"
                        x-model="coeff_a0_str"
                        error-model="coeff_a0_error"
                    />
                    <x-app-ui.input-text
                        label="Coeficiente aₙ"
                        name="coeff_an"
                        placeholder="Ej: (2/(n*pi))*sin(n*pi/2)"
                        x-model="coeff_an_str"
                        error-model="coeff_an_error"
                    />
                    <x-app-ui.input-text
                        label="Coeficiente bₙ"
                        name="coeff_bn"
                        placeholder="Ej: 0"
                        x-model="coeff_bn_str"
                        error-model="coeff_bn_error"
                    />
                </div>

                <div class="flex">
                    <x-app-ui.button class="h-12" type="button" @click="calculateAndRedraw()" loading-text="Calculando...">
                        Calcular
                    </x-app-ui.button>
                </div>
            </div>
        </x-slot>
    </x-three-column-tool>
</div>