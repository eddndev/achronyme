<x-guest-layout title="Reestablecer contraseña">
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
            ¿Recuerdas tu contraseña?
            <a href="{{ route('login') }}" class="font-medium text-purple-blue-700 hover:text-purple-blue-800 dark:text-purple-blue-300 dark:hover:text-purple-blue-400">Inicia Sesión</a>
            </p>
        </div>

        <form method="POST" action="{{ route('password.store') }}" class="space-y-6 mt-6">
            @csrf

            <!-- Password Reset Token -->
            <input type="hidden" name="token" value="{{ $request->route('token') }}">

            <!-- Email Address -->
            <div>
                <x-input-label for="email" :value="__('Email')" />
                <div class="mt-2">
                    <x-text-input id="email" class="block mt-1 w-full" type="email" name="email" :value="old('email', $request->email)" required autofocus autocomplete="username" />
                </div>
                <x-input-error :messages="$errors->get('email')" class="mt-2" />
            </div>

            <!-- Password -->
            <div>
                <x-input-label for="password" :value="__('Password')" />
                <div class="mt-2">
                    <x-text-input id="password" class="block mt-1 w-full" type="password" name="password" required autocomplete="new-password" />
                </div>
                <x-input-error :messages="$errors->get('password')" class="mt-2" />
            </div>

            <!-- Confirm Password -->
            <div>
                <x-input-label for="password_confirmation" :value="__('Confirm Password')" />
                <div class="mt-2">
                    <x-text-input id="password_confirmation" class="block mt-1 w-full"
                                        type="password"
                                        name="password_confirmation" required autocomplete="new-password" />
                </div>
                <x-input-error :messages="$errors->get('password_confirmation')" class="mt-2" />
            </div>

            <div class="flex items-center justify-end">
                <x-primary-button>
                    {{ __('Reset Password') }}
                </x-primary-button>
            </div>
        </form>
    </div>
</x-guest-layout>
