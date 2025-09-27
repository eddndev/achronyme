<x-guest-layout title="Verifica tu correo">
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
                {{ __('Thanks for signing up! Before getting started, could you verify your email address by clicking on the link we just emailed to you? If you didn\'t receive the email, we will gladly send you another.') }}
            </p>
        </div>

        @if (session('status') == 'verification-link-sent')
            <div class="mt-6 font-medium text-sm text-success">
                {{ __('A new verification link has been sent to the email address you provided during registration.') }}
            </div>
        @endif

        <div class="mt-6 flex items-center justify-between">
            <form method="POST" action="{{ route('verification.send') }}">
                @csrf

                <div>
                    <x-primary-button>
                        {{ __('Resend Verification Email') }}
                    </x-primary-button>
                </div>
            </form>

            <form method="POST" action="{{ route('logout') }}">
                @csrf

                <button type="submit" class="text-purple-blue-700 hover:text-purple-blue-800 dark:text-purple-blue-300 dark:hover:text-purple-blue-400">
                    {{ __('Log Out') }}
                </button>
            </form>
        </div>
    </div>
</x-guest-layout>
