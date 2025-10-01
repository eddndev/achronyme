<?php

namespace App\Http\Controllers;

use Illuminate\Http\Request;
use Illuminate\View\View;

class ToolController extends Controller
{
    /**
     * Display the fourier series tool page.
     */
    public function fourierSeries(): View
    {
        return view('tools.fourier.fs.index');
    }

    /**
     * Display the fourier transform tool page.
     */
    public function fourierTransform(): View
    {
        return view('ft.index');
    }

    /**
     * Display the convolution tool page.
     */
    public function convolution(): View
    {
        return view('conv.index');
    }
}
