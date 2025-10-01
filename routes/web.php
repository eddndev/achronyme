<?php

use App\Http\Controllers\HomeController;
use App\Http\Controllers\ProfileController;
use App\Http\Controllers\ToolController;
use Illuminate\Support\Facades\Route;

Route::get('/', [HomeController::class, 'index'])->name('home');

Route::get('/fourier-series', [ToolController::class, 'fourierSeries'])->name('fourier-series');
Route::get('/fourier-transform', [ToolController::class, 'fourierTransform'])->name('fourier-transform');
Route::get('/convolution', [ToolController::class, 'convolution'])->name('convolution');

Route::get('/dashboard', function () {
    return view('dashboard');
})->middleware(['auth', 'verified'])->name('dashboard');

Route::middleware('auth')->group(function () {
    Route::get('/profile', [ProfileController::class, 'edit'])->name('profile.edit');
    Route::patch('/profile', [ProfileController::class, 'update'])->name('profile.update');
    Route::delete('/profile', [ProfileController::class, 'destroy'])->name('profile.destroy');
});

Route::middleware('web')->group(function () {
    require __DIR__.'/auth.php';
    require __DIR__.'/socialite.php';
});