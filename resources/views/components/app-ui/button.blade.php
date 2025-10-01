@props([
    'type' => 'submit',
    'loadingText' => 'Cargando...'
])

<button {{ $attributes->merge([
    'type' => $type,
    'class' => '
        w-full relative group overflow-hidden isolate
        inline-flex items-center justify-center rounded-md px-4 py-2 text-sm font-semibold text-white
        transition-all duration-300
        focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-blue-500
        dark:focus:ring-offset-slate-800
        btn-purple-blue
    '
    ]) }} 
    :disabled="isLoading"
>
    <div x-show="isLoading" class="flex items-center justify-center">
        <x-app-ui.loading-spinner class="h-5 w-5 mr-2" />
        <span x-text="'{{ $loadingText }}'"></span>
    </div>
    <span x-show="!isLoading">{{ $slot }}</span>
</button>
