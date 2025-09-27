@props(['value'])

<label {{ $attributes->merge(['class' => 'block text-sm/6 font-medium text-slate-800 dark:text-slate-200']) }}>
    {{ $value ?? $slot }}
</label>
