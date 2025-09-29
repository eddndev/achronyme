@props([
    'label' => '',
    'functionAddon' => 'f(t)',

    'functionName',
    'functionId' => null,
    'functionPlaceholder' => '',
    'functionValue' => '',

    'domainStartName',
    'domainStartId' => null,
    'domainStartPlaceholder' => 'a',
    'domainStartValue' => '',

    'domainEndName',
    'domainEndId' => null,
    'domainEndPlaceholder' => 'b',
    'domainEndValue' => '',
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
        <div class="flex">
            <div class="{{ $addonClasses }} rounded-tl-md">{{ $functionAddon }}</div>
            <input
                type="text"
                name="{{ $functionName }}"
                id="{{ $funcId }}"
                placeholder="{{ $functionPlaceholder }}"
                value="{{ $functionValue }}"
                class="-ml-px {{ $inputClasses }} rounded-tr-md"
                {{ $attributes->whereStartsWith('wire:function') }}
            />
        </div>

        {{-- Bottom Row: Domain [a, b] inputs --}}
        <div class="flex -mt-px">
            <input
                type="text"
                name="{{ $domainStartName }}"
                id="{{ $startId }}"
                placeholder="{{ $domainStartPlaceholder }}"
                value="{{ $domainStartValue }}"
                class="{{ $inputClasses }} rounded-bl-md"
                {{ $attributes->whereStartsWith('wire:domainStart') }}
            />
            <input
                type="text"
                name="{{ $domainEndName }}"
                id="{{ $endId }}"
                placeholder="{{ $domainEndPlaceholder }}"
                value="{{ $domainEndValue }}"
                class="-ml-px {{ $inputClasses }} rounded-br-md"
                {{ $attributes->whereStartsWith('wire:domainEnd') }}
            />
        </div>
    </div>
</div>
