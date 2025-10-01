@props([
    'label' => '',
    'functionAddon' => 'f(t)',

    'functionName',
    'functionId' => null,
    'functionPlaceholder' => '',
    'functionErrorModel' => null,

    'domainStartName',
    'domainStartId' => null,
    'domainStartPlaceholder' => 'a',
    'domainStartErrorModel' => null,

    'domainEndName',
    'domainEndId' => null,
    'domainEndPlaceholder' => 'b',
    'domainEndErrorModel' => null,
])

@php
    $funcId = $functionId ?? $functionName;
    $startId = $domainStartId ?? $domainStartName;
    $endId = $domainEndId ?? $domainEndName;

    $inputClasses = 'block w-full grow bg-white px-3 py-1.5 text-base text-slate-900 outline-1 -outline-offset-1 outline-slate-300 placeholder:text-slate-400 focus:outline-2 focus:-outline-offset-2 focus:outline-purple-blue-600 sm:text-sm/6 dark:bg-white/5 dark:text-white dark:outline-slate-700 dark:placeholder:text-slate-500 dark:focus:outline-purple-blue-500';
    $addonClasses = 'flex shrink-0 items-center bg-white px-3 text-base text-slate-500 outline-1 -outline-offset-1 outline-slate-300 sm:text-sm/6 dark:bg-white/5 dark:text-slate-400 dark:outline-slate-700';
@endphp

<div>
    @if ($label)
        <label for="{{ $funcId }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
    @endif

    <div class="@if($label) mt-2 @endif">
        {{-- Top Row: f(t) input --}}
        <div class="flex" x-data="{ funcError: {{ $functionErrorModel ?? 'null' }} }">
            <div class="{{ $addonClasses }} rounded-tl-md" :class="funcError ? 'outline-red-300 dark:outline-red-700 text-red-500 dark:text-red-400' : ''">{{ $functionAddon }}</div>
            <input
                type="text"
                name="{{ $functionName }}"
                id="{{ $funcId }}"
                placeholder="{{ $functionPlaceholder }}"
                x-model="functionDefinition"
                class="-ml-px {{ $inputClasses }} rounded-tr-md"
                :class="funcError ? 'outline-red-300 focus:outline-red-600 dark:outline-red-700 dark:focus:outline-red-500' : ''"
            />
            <template x-if="funcError">
                <svg viewBox="0 0 16 16" fill="currentColor" data-slot="icon" aria-hidden="true" class="pointer-events-none col-start-1 row-start-1 mr-3 size-5 self-center justify-self-end text-red-500 sm:size-4 dark:text-red-400">
                <path d="M8 15A7 7 0 1 0 8 1a7 7 0 0 0 0 14ZM8 4a.75.75 0 0 1 .75.75v3a.75.75 0 0 1-1.5 0v-3A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" fill-rule="evenodd" />
                </svg>
            </template>
        </div>
        <template x-if="{{ $functionErrorModel ?? 'null' }}">
            <p id="{{ $funcId }}-error" class="mt-1 text-sm text-red-600 dark:text-red-400" x-text="{{ $functionErrorModel }}"></p>
        </template>

        {{-- Bottom Row: Domain [a, b] inputs --}}
        <div class="flex -mt-px">
            <div class="grow" x-data="{ startError: {{ $domainStartErrorModel ?? 'null' }} }">
                <input
                    type="text"
                    name="{{ $domainStartName }}"
                    id="{{ $startId }}"
                    placeholder="{{ $domainStartPlaceholder }}"
                    x-model="domainStart"
                    class="{{ $inputClasses }} rounded-bl-md"
                    :class="startError ? 'outline-red-300 focus:outline-red-600 dark:outline-red-700 dark:focus:outline-red-500' : ''"
                />
                <template x-if="startError">
                    <p id="{{ $startId }}-error" class="mt-1 text-sm text-red-600 dark:text-red-400" x-text="startError"></p>
                </template>
            </div>
            <div class="grow -ml-px" x-data="{ endError: {{ $domainEndErrorModel ?? 'null' }} }">
                <input
                    type="text"
                    name="{{ $domainEndName }}"
                    id="{{ $endId }}"
                    placeholder="{{ $domainEndPlaceholder }}"
                    x-model="domainEnd"
                    class="{{ $inputClasses }} rounded-br-md"
                    :class="endError ? 'outline-red-300 focus:outline-red-600 dark:outline-red-700 dark:focus:outline-red-500' : ''"
                />
                <template x-if="endError">
                    <p id="{{ $endId }}-error" class="mt-1 text-sm text-red-600 dark:text-red-400" x-text="endError"></p>
                </template>
            </div>
        </div>
    </div>
</div>
