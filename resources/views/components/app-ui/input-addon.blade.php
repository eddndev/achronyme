@props([
    'label' => '',
    'addon',
    'position' => 'left', // 'left' or 'right'
    'type' => 'text',
    'name',
    'id' => null,
    'placeholder' => '',
    'value' => ''
])

@php
    $inputId = $id ?? $name;
    $baseInputClasses = 'block w-full grow bg-white px-3 py-1.5 text-base text-slate-900 outline-1 -outline-offset-1 outline-slate-300 placeholder:text-slate-400 focus:outline-2 focus:-outline-offset-2 focus:outline-purple-blue-600 sm:text-sm/6 dark:bg-white/5 dark:text-white dark:outline-slate-700 dark:placeholder:text-slate-500 dark:focus:outline-purple-blue-500';
    $addonClasses = 'flex shrink-0 items-center px-3 text-base text-slate-500 sm:text-sm/6 bg-white dark:bg-white/5 dark:text-slate-400 outline-1 -outline-offset-1 outline-slate-300 dark:outline-slate-700';
@endphp

<div>
    @if ($label)
        <label for="{{ $inputId }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
    @endif
    <div class="@if($label) mt-2 @endif flex">
        @if ($position === 'left')
            <div class="{{ $addonClasses }} rounded-l-md">{{ $addon }}</div>
            <input
                type="{{ $type }}"
                name="{{ $name }}"
                id="{{ $inputId }}"
                placeholder="{{ $placeholder }}"
                value="{{ $value }}"
                {{ $attributes->merge(['class' => "-ml-px rounded-r-md " . $baseInputClasses]) }}
            />
        @else
            <input
                type="{{ $type }}"
                name="{{ $name }}"
                id="{{ $inputId }}"
                placeholder="{{ $placeholder }}"
                value="{{ $value }}"
                {{ $attributes->merge(['class' => "rounded-l-md " . $baseInputClasses]) }}
            />
            <div class="-ml-px {{ $addonClasses }} rounded-r-md">{{ $addon }}</div>
        @endif
    </div>
</div>