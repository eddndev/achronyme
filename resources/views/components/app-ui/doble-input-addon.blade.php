@props([
    'label' => '',
    'addon',
    'name1',
    'id1' => null,
    'type1' => 'text',
    'placeholder1' => '',
    'value1' => '',
    'name2',
    'id2' => null,
    'type2' => 'text',
    'placeholder2' => '',
    'value2' => '',
])

@php
    $inputId1 = $id1 ?? $name1;
    $inputId2 = $id2 ?? $name2;
    $inputClasses = 'block w-full grow bg-white px-3 py-1.5 text-base text-slate-900 outline-1 -outline-offset-1 outline-slate-300 placeholder:text-slate-400 focus:outline-2 focus:-outline-offset-2 focus:outline-purple-blue-600 sm:text-sm/6 dark:bg-white/5 dark:text-white dark:outline-slate-700 dark:placeholder:text-slate-500 dark:focus:outline-purple-blue-500';
    $addonClasses = '-ml-px flex shrink-0 items-center bg-white px-3 text-base text-slate-500 outline-1 -outline-offset-1 outline-slate-300 sm:text-sm/6 dark:bg-white/5 dark:text-slate-400 dark:outline-slate-700';
@endphp

<div {{ $attributes }}>
    @if ($label)
        <label for="{{ $inputId1 }}" class="block text-sm/6 font-medium text-slate-900 dark:text-white">{{ $label }}</label>
    @endif
    <div class="@if($label) mt-2 @endif flex">
        <input
            type="{{ $type1 }}"
            name="{{ $name1 }}"
            id="{{ $inputId1 }}"
            placeholder="{{ $placeholder1 }}"
            value="{{ $value1 }}"
            class="{{ $inputClasses }} rounded-l-md"
        />
        <div class="{{ $addonClasses }}">{{ $addon }}</div>
        <input
            type="{{ $type2 }}"
            name="{{ $name2 }}"
            id="{{ $inputId2 }}"
            placeholder="{{ $placeholder2 }}"
            value="{{ $value2 }}"
            class="-ml-px {{ $inputClasses }} rounded-r-md"
        />
    </div>
</div>