<x-guest-layout title="¿Olvidaste tu contraseña?">
    <x-image.logo
        class="mx-auto h-80 w-auto absolute top-1/2 left-1/2 -translate-y-1/2 -translate-x-1/2 -z-10"
        src="resources/images/logo.png"
        alt="Logotipo de Achronyme"
        size="md"
        :priority="true"
    />
    <div class="border border-slate-200 bg-slate-100/75 px-6 py-12 shadow-md backdrop-blur-lg sm:rounded-lg sm:px-12 dark:border-slate-700 dark:bg-slate-900/75">
        <div>
            <h2 class="mt-4 text-2xl/9 font-bold tracking-tight text-gray-900 dark:text-white">{{ $title }}</h2>
            <p class="mt-2 text-sm/6 text-slate-500 dark:text-slate-400">
                {{ __('Forgot your password? No problem. Just let us know your email address and we will email you a password reset link that will allow you to choose a new one.') }}
            </p>
            <p class="mt-2 text-sm/6 text-slate-500 dark:text-slate-400">
                ¿Recuerdas tu contraseña?
                <a href="{{ route('login') }}" class="font-medium text-purple-blue-700 hover:text-purple-blue-800 dark:text-purple-blue-300 dark:hover:text-purple-blue-400">Inicia sesión</a>
            </p>
        </div>

        <!-- Session Status -->
        <x-auth-session-status class="mt-6" :status="session('status')" />

        <form method="POST" action="{{ route('password.email') }}" class="space-y-6 mt-6">
            @csrf

            <!-- Email Address -->
            <div>
                <x-input-label for="email" :value="__('Email')" />
                <div class="mt-2">
                    <x-text-input id="email" class="block mt-1 w-full" type="email" name="email" :value="old('email')" required autofocus />
                </div>
                <x-input-error :messages="$errors->get('email')" class="mt-2" />
            </div>

            <div class="flex items-center justify-end">
                <x-primary-button>
                    {{ __('Email Password Reset Link') }}
                </x-primary-button>
            </div>
        </form>
    </div>
</x-guest-layout>
