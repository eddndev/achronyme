<button {{ $attributes->merge([
    'type' => 'submit',
    'class' => '
        w-full relative group overflow-hidden isolate
        inline-flex items-center justify-center rounded-md px-4 py-2 text-sm font-semibold text-white
        transition-all duration-300
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-blue-500
        dark:focus:ring-offset-slate-800
        btn-purple-blue
    '
    ]) }}
>
    {{-- El span es crucial para que el texto aparezca por encima del efecto de brillo --}}
    <span class="z-10">{{ $slot }}</span>
</button>
