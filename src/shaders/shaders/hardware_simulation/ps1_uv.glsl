    {
        const float PS1_GRID = 140.0;
        vec2 ps_grid = floor(v_uv * PS1_GRID);
        v_uv = (ps_grid + 0.5) / PS1_GRID;
    }
