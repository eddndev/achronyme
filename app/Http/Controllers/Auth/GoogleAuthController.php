<?php

namespace App\Http\Controllers\Auth;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Hash;
use Illuminate\Support\Facades\Log; // Para registrar errores
use Illuminate\Support\Str;
use Laravel\Socialite\Facades\Socialite;
use Exception; // Para capturar excepciones

class GoogleAuthController extends Controller
{
    /**
     * Redirige al usuario a la página de autenticación de Google.
     */
    public function redirect()
    {
        return Socialite::driver('google')->redirect();
    }

    /**
     * Obtiene la información del usuario de Google y maneja el login/registro.
     */
    public function callback()
    {
        try {
            $googleUser = Socialite::driver('google')->user();

            // Busca si el usuario ya existe por el ID de Google
            $user = User::where('google_id', $googleUser->getId())->first();

            if ($user) {
                // Si existe, actualiza el token y listo
                $user->update([
                    'google_token' => $googleUser->token,
                    'google_refresh_token' => $googleUser->refreshToken,
                ]);
            } else {
                // Si no existe por ID de Google, busca por email
                $user = User::where('email', $googleUser->getEmail())->first();

                if ($user) {
                    // Si existe por email, vincula la cuenta con el ID de Google
                    $user->update([
                        'google_id' => $googleUser->getId(),
                        'google_token' => $googleUser->token,
                        'google_refresh_token' => $googleUser->refreshToken,
                    ]);
                } else {
                    // Si no existe de ninguna forma, crea un nuevo usuario
                    $user = User::create([
                        'name' => $googleUser->getName(),
                        'email' => $googleUser->getEmail(),
                        'google_id' => $googleUser->getId(),
                        'google_token' => $googleUser->token,
                        'google_refresh_token' => $googleUser->refreshToken,
                        'email_verified_at' => now(),
                        'password' => Hash::make(Str::random(24)), // Contraseña aleatoria segura
                    ]);
                }
            }

            // Inicia sesión con el usuario encontrado o creado
            Auth::login($user, remember: true);

            // Redirige al usuario a su dashboard
            return redirect()->intended(route('home', absolute: false));

        } catch (Exception $e) {
            // Si algo sale mal (ej. el usuario deniega el acceso),
            // se registra el error y se redirige al login con un mensaje.
            Log::error('Error en la autenticación con Google', ['exception' => $e]);

            return redirect('/login')->with('error', 'No se pudo autenticar con Google. Por favor, inténtelo de nuevo.');
        }
    }
}