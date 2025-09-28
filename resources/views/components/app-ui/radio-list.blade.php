@props([
    'legend' => '',
    'name',
    'options' => [],
    'checkedValue' => null
])

<fieldset aria-label="{{ $legend }}" {{ $attributes->merge(['class' => '-space-y-px rounded-md bg-white dark:bg-slate-900/50']) }}>
    @if($legend)
        <legend class="sr-only">{{ $legend }}</legend>
    @endif

    @foreach ($options as $option)
        <label
            aria-label="{{ $option['title'] }}"
            aria-description="{{ $option['description'] }}"
            class="group flex cursor-pointer border border-slate-200 p-4 first:rounded-tl-md first:rounded-tr-md last:rounded-br-md last:rounded-bl-md focus:outline-hidden has-checked:relative has-checked:border-purple-blue-200 has-checked:bg-purple-blue-50 dark:border-slate-700 dark:has-checked:border-purple-blue-800 dark:has-checked:bg-purple-blue-500/10"
        >
            <input
                type="radio"
                name="{{ $name }}"
                value="{{ $option['value'] }}"
                @if ($checkedValue === $option['value']) checked @endif
                class="relative mt-0.5 size-4 shrink-0 appearance-none rounded-full border border-slate-300 bg-white before:absolute before:inset-1 before:rounded-full before:bg-white not-checked:before:hidden checked:border-purple-blue-600 checked:bg-purple-blue-600 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-purple-blue-600 disabled:border-slate-300 disabled:bg-slate-100 disabled:before:bg-slate-400 dark:border-white/10 dark:bg-white/5 dark:checked:border-purple-blue-500 dark:checked:bg-purple-blue-500 dark:focus-visible:outline-purple-blue-500 dark:disabled:border-white/5 dark:disabled:bg-white/10 dark:disabled:before:bg-white/20 forced-colors:appearance-auto forced-colors:before:hidden"
            />
            <span class="ml-3 flex flex-col">
                <span class="block text-sm font-medium text-slate-900 group-has-checked:text-purple-blue-900 dark:text-white dark:group-has-checked:text-purple-blue-300">{{ $option['title'] }}</span>
                <span class="block text-sm text-slate-500 group-has-checked:text-purple-blue-700 dark:text-slate-400 dark:group-has-checked:text-purple-blue-300/75">{{ $option['description'] }}</span>
            </span>
        </label>
    @endforeach
</fieldset>