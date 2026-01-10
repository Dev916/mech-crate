<?php

namespace App\Http\Controllers;

use Inertia\Inertia;
use Inertia\Response;

class DashboardController extends Controller
{
    /**
     * Display the dashboard.
     */
    public function __invoke(): Response
    {
        return Inertia::render('Dashboard', [
            'user' => auth()->user(),
        ]);
    }
}
