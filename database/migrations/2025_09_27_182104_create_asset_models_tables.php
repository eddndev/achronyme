<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     * Crea todas las tablas necesarias para los modelos de activos del sitio
     * (LogoAsset, MediumAsset, FullAsset) en una sola operación.
     */
    public function up(): void
    {
        // Tabla para logos e imágenes pequeñas
        Schema::create('logo_assets', function (Blueprint $table) {
            $table->id();
            $table->string('original_path')->unique()->comment('Ruta relativa al recurso original, ej: "brand/logo.png"');
            $table->string('title')->nullable();
            $table->string('alt')->nullable();
            $table->string('collection')->default('default')->comment('Colección de Spatie Media Library');
            $table->json('meta')->nullable()->comment('Datos adicionales en formato JSON');
            $table->timestamps();
        });

        // Tabla para imágenes de tamaño mediano
        Schema::create('medium_assets', function (Blueprint $table) {
            $table->id();
            $table->string('original_path')->unique()->comment('Ruta relativa al recurso original, ej: "banners/hero.jpg"');
            $table->string('title')->nullable();
            $table->string('alt')->nullable();
            $table->string('collection')->default('default');
            $table->json('meta')->nullable();
            $table->timestamps();
        });

        // Tabla para imágenes de alta resolución
        Schema::create('full_assets', function (Blueprint $table) {
            $table->id();
            $table->string('original_path')->unique()->comment('Ruta relativa al recurso original, ej: "covers/landing-hq.jpg"');
            $table->string('title')->nullable();
            $table->string('alt')->nullable();
            $table->string('collection')->default('default');
            $table->json('meta')->nullable();
            $table->timestamps();
        });
    }

    /**
     * Reverse the migrations.
     * Elimina las tres tablas de activos.
     */
    public function down(): void
    {
        Schema::dropIfExists('logo_assets');
        Schema::dropIfExists('medium_assets');
        Schema::dropIfExists('full_assets');
    }
};