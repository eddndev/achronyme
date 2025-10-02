{{-- Fourier Transform Tool Interface --}}
<div x-data="fourierTransformState()" x-init="init()">
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
                                    <x-app-ui.checkbox id="render_time_domain" value="time" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_time_domain" class="font-medium text-slate-900 dark:text-white">Dominio del tiempo</label>
                                    <p class="text-slate-500 dark:text-slate-400">Muestra f(t).</p>
                                </div>
                            </div>

                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_magnitude" value="magnitude" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_magnitude" class="font-medium text-slate-900 dark:text-white">Magnitud |F(ω)|</label>
                                    <p class="text-slate-500 dark:text-slate-400">Espectro de magnitud.</p>
                                </div>
                            </div>

                            <div class="relative flex items-start">
                                <div class="flex h-6 items-center">
                                    <x-app-ui.checkbox id="render_phase" value="phase" x-model="renderOptions" />
                                </div>
                                <div class="ml-3 text-sm leading-6">
                                    <label for="render_phase" class="font-medium text-slate-900 dark:text-white">Fase ∠F(ω)</label>
                                    <p class="text-slate-500 dark:text-slate-400">Espectro de fase.</p>
                                </div>
                            </div>
                        </div>
                    </fieldset>
                </div>

                <div>
                    <x-app-ui.slider
                        label="Rango de frecuencia (ω)"
                        name="omega_range"
                        min="1"
                        max="50"
                        step="1"
                        value="20"
                        x-model="omegaRange"
                    />
                </div>

                <div>
                    <x-app-ui.slider
                        label="Resolución de muestreo"
                        name="sampling_resolution"
                        min="100"
                        max="2000"
                        step="100"
                        value="500"
                        x-model="samplingResolution"
                    />
                </div>
            </div>
        </x-slot>

        {{-- Center: Charts --}}
        <div class="space-y-6">
            {{-- Time Domain Chart --}}
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <h3 class="text-sm font-medium text-slate-900 dark:text-white mb-3">Dominio del Tiempo - f(t)</h3>
                <div class="relative w-full" style="height: 300px; max-width: 100%;" id="timeDomainContainer">
                    <canvas id="timeDomainChart"></canvas>
                </div>
            </div>

            
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <h3 class="text-sm font-medium text-slate-900 dark:text-white mb-3">Magnitud |F(ω)|</h3>
                <div class="relative w-full" style="height: 250px;" id="magnitudeContainer">
                    <canvas id="magnitudeChart"></canvas>
                </div>
            </div>

            {{-- Phase Spectrum --}}
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <h3 class="text-sm font-medium text-slate-900 dark:text-white mb-3">Fase ∠F(ω)</h3>
                <div class="relative w-full" style="height: 250px;" id="phaseContainer">
                    <canvas id="phaseChart"></canvas>
                </div>
            </div>
        </div>

        {{-- Right Sidebar: Function Inputs --}}
        <x-slot:rightSidebar>
            <div class="space-y-6">
                <div>
                    <h3 class="font-medium text-slate-900 dark:text-white mb-4">Función en el Dominio del Tiempo</h3>
                    <p class="text-sm text-slate-500 dark:text-slate-400 mb-4">
                        Define f(t) por tramos. Puedes agregar múltiples funciones para crear una función por partes continua.
                    </p>

                    {{-- Function Inputs (Dynamic) --}}
                    <div class="space-y-4">
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
                                    function-placeholder="Ej: exp(-abs(t))"
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
                            <x-app-ui.secondary-button type="button" @click.prevent="addFunction()">
                                <div class="flex space-x-2 items-center">
                                    <svg class="size-5 -ml-0.5" fill="currentColor"><use href="#icon-plus"></use></svg>
                                    <div>Agregar función</div>
                                </div>
                            </x-app-ui.secondary-button>
                        </div>
                    </div>
                </div>

                {{-- Calculate Button --}}
                <div class="pt-4 border-t border-slate-200 dark:border-slate-700">
                    <x-app-ui.button class="w-full h-12" type="button" @click="calculateTransform()" loading-text="Calculando...">
                        <div class="flex items-center justify-center space-x-2">
                            <svg class="size-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
                            </svg>
                            <span>Calcular Transformada</span>
                        </div>
                    </x-app-ui.button>
                </div>
            </div>
        </x-slot>
    </x-three-column-tool>
</div>