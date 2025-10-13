{{-- Convolution Tool Interface --}}
<div x-data="convolutionState()" x-init="init()">
    <x-three-column-tool>
        {{-- Left Sidebar: Visualization Options --}}
        <x-slot:leftSidebar>
            <div class="space-y-6">
                <div>
                    <h3 class="text-sm font-medium leading-6 text-slate-900 dark:text-white mb-2">Visualización</h3>
                    <fieldset>
                        <legend class="sr-only">Opciones de gráficas</legend>
                        <div class="space-y-4">
                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_f" value="f" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_f" class="font-medium text-slate-900 dark:text-white">Mostrar f(τ)</label>
                                    <p class="text-slate-500 dark:text-slate-400">Primera función.</p>
                                </div>
                            </div>

                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_g" value="g" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_g" class="font-medium text-slate-900 dark:text-white">Mostrar g(t-τ)</label>
                                    <p class="text-slate-500 dark:text-slate-400">Función desplazada.</p>
                                </div>
                            </div>

                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_product" value="product" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_product" class="font-medium text-slate-900 dark:text-white">Mostrar producto f·g</label>
                                    <p class="text-slate-500 dark:text-slate-400">Área del producto.</p>
                                </div>
                            </div>

                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_result" value="result" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_result" class="font-medium text-slate-900 dark:text-white">Mostrar (f*g)(t)</label>
                                    <p class="text-slate-500 dark:text-slate-400">Resultado final.</p>
                                </div>
                            </div>
                        </div>
                    </fieldset>
                </div>

                <div class="pt-4 border-t border-slate-200 dark:border-slate-700">
                    <x-app-ui.slider
                        label="Tiempo t"
                        name="current_time"
                        step="0.1"
                        value="0"
                        x-model.debounce.16ms="currentTime"
                        ::min="tInitialNumeric"
                        ::max="tFinalNumeric"
                    />
                </div>

                <div class="pt-4 border-t border-slate-200 dark:border-slate-700">
                    <div class="relative flex items-start mb-4">
                        <div class="flex h-6 items-center">
                            <x-app-ui.checkbox id="manual_range" value="manual" x-model="manualRange" />
                        </div>
                        <div class="ml-3 text-sm leading-6">
                            <label for="manual_range" class="font-medium text-slate-900 dark:text-white">Rango manual</label>
                            <p class="text-slate-500 dark:text-slate-400">Define t_inicial y t_final.</p>
                        </div>
                    </div>

                    <div x-show="manualRange" x-transition class="space-y-4">
                        <x-app-ui.input-text
                            label="t inicial"
                            name="t_initial"
                            placeholder="-10"
                            x-model="tInitial"
                            error-model="tInitial_error"
                        />

                        <x-app-ui.input-text
                            label="t final"
                            name="t_final"
                            placeholder="10"
                            x-model="tFinal"
                            error-model="tFinal_error"
                        />
                    </div>
                </div>
            </div>
        </x-slot>

        {{-- Center: Charts --}}
        <div class="space-y-6">
            {{-- Functions Chart (f and g) --}}
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <h3 class="text-sm font-medium text-slate-900 dark:text-white mb-3">Funciones f(τ) y g(t-τ)</h3>
                <div class="relative w-full" style="height: 300px; max-width: 100%;" id="functionsContainer">
                    <canvas id="functionsChart"></canvas>
                </div>
            </div>

            {{-- Convolution Result Chart --}}
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <h3 class="text-sm font-medium text-slate-900 dark:text-white mb-3">Resultado (f*g)(t)</h3>
                <div class="relative w-full" style="height: 300px;" id="resultContainer">
                    <canvas id="resultChart"></canvas>
                </div>
            </div>
        </div>

        {{-- Right Sidebar: Function Inputs --}}
        <x-slot:rightSidebar>
            <div class="space-y-6">
                <div>
                    <h3 class="font-medium text-slate-900 dark:text-white mb-4">Función f(t)</h3>
                    <p class="text-sm text-slate-500 dark:text-slate-400 mb-4">
                        Define f(t) por tramos.
                    </p>

                    {{-- Function f(t) Inputs (Dynamic) --}}
                    <div class="space-y-4">
                        <template x-for="(func, index) in functionsF" :key="func.id">
                            <div class="relative">
                                <template x-if="functionsF.length > 1">
                                    <div class="absolute -top-2 -right-2 z-10">
                                        <x-app-ui.danger-circular-button
                                            icon="trash"
                                            @click.prevent="removeFunctionF(func.id)"
                                            title="Eliminar función"
                                        />
                                    </div>
                                </template>
                                <x-app-ui.function-domain
                                    x-model-function="func.definition"
                                    x-model-domain-start="func.domainStart"
                                    x-model-domain-end="func.domainEnd"
                                    function-placeholder="Ej: exp(-t)"
                                    domain-start-placeholder="-5"
                                    domain-end-placeholder="5"
                                    :function-error-model="'func.definitionError'"
                                    :domain-start-error-model="'func.domainStartError'"
                                    :domain-end-error-model="'func.domainEndError'"
                                    ::is-domain-start-disabled="index > 0"
                                    ::index="index"
                                />
                            </div>
                        </template>

                        {{-- Add Function Button --}}
                        <div class="pt-2">
                            <x-app-ui.secondary-button type="button" @click.prevent="addFunctionF()">
                                <div class="flex space-x-2 items-center">
                                    <svg class="size-5 -ml-0.5" fill="currentColor"><use href="#icon-plus"></use></svg>
                                    <div>Agregar tramo</div>
                                </div>
                            </x-app-ui.secondary-button>
                        </div>
                    </div>
                </div>

                <div class="pt-4 border-t border-slate-200 dark:border-slate-700">
                    <h3 class="font-medium text-slate-900 dark:text-white mb-4">Función g(t)</h3>
                    <p class="text-sm text-slate-500 dark:text-slate-400 mb-4">
                        Define g(t) por tramos.
                    </p>

                    {{-- Function g(t) Inputs (Dynamic) --}}
                    <div class="space-y-4">
                        <template x-for="(func, index) in functionsG" :key="func.id">
                            <div class="relative">
                                <template x-if="functionsG.length > 1">
                                    <div class="absolute -top-2 -right-2 z-10">
                                        <x-app-ui.danger-circular-button
                                            icon="trash"
                                            @click.prevent="removeFunctionG(func.id)"
                                            title="Eliminar función"
                                        />
                                    </div>
                                </template>
                                <x-app-ui.function-domain
                                    x-model-function="func.definition"
                                    x-model-domain-start="func.domainStart"
                                    x-model-domain-end="func.domainEnd"
                                    function-placeholder="Ej: rect(t/2)"
                                    domain-start-placeholder="-1"
                                    domain-end-placeholder="1"
                                    :function-error-model="'func.definitionError'"
                                    :domain-start-error-model="'func.domainStartError'"
                                    :domain-end-error-model="'func.domainEndError'"
                                    ::is-domain-start-disabled="index > 0"
                                    ::index="index"
                                />
                            </div>
                        </template>

                        {{-- Add Function Button --}}
                        <div class="pt-2">
                            <x-app-ui.secondary-button type="button" @click.prevent="addFunctionG()">
                                <div class="flex space-x-2 items-center">
                                    <svg class="size-5 -ml-0.5" fill="currentColor"><use href="#icon-plus"></use></svg>
                                    <div>Agregar tramo</div>
                                </div>
                            </x-app-ui.secondary-button>
                        </div>
                    </div>
                </div>

                {{-- Calculate Button --}}
                <div class="pt-4 border-t border-slate-200 dark:border-slate-700">
                    <x-app-ui.button class="w-full h-10" type="button" @click="calculateConvolution()" loading-text="Calculando...">
                        <div class="flex items-center justify-center space-x-2">
                            
                            <span>Calcular Convolución</span>
                        </div>
                    </x-app-ui.button>
                </div>
            </div>
        </x-slot>
    </x-three-column-tool>
</div>
