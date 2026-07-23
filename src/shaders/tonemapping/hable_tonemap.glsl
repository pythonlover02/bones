    {
        const float HABLE_W_INV = 1.0 / 0.81243248;
        c = clamp(hable_map(c * 2.0) * HABLE_W_INV, 0.0, 1.0);
    }
