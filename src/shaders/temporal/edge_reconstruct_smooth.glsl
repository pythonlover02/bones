    {
        float egx = dot(grad_x_rgb, LUMA_BT709);
        float egy = dot(grad_y_rgb, LUMA_BT709);
        float egm2 = egx * egx + egy * egy;
        vec2 egd = vec2(-egy, egx) / max(sqrt(egm2), 0.0001);
        vec3 edp = texture(u_input, v_uv + egd * inv).rgb;
        vec3 edn = texture(u_input, v_uv - egd * inv).rgb;
        vec3 eda = grade((edp + edn) * 0.5, v_uv, frag_coord, res_scale);
        float edw = clamp(egm2 * 4.0, 0.0, 1.0);
        vec3 esp = mix(c, eda, edw * 0.3);
        float el = dot(c, LUMA_AVG);
        float elt = el - dot(history, LUMA_AVG);
        float edn2 = lgrad_x * lgrad_x * 0.25 + lgrad_y * lgrad_y * 0.25 + 0.001;
        vec2 ef = clamp(vec2(lgrad_x * 0.5, lgrad_y * 0.5) * (-elt / edn2),
                        vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
        vec3 ew_ = clamp(texture(u_history, v_uv + ef * inv).rgb, min(dmin_x, c), max(dmax_x, c));
        vec3 edf = abs(esp - ew_);
        float ecf = 1.0 - clamp(max(edf.r, max(edf.g, edf.b)) * 10.0, 0.0, 1.0);
        float eoe = clamp(egm2 * 16.0, 0.0, 1.0);
        float ewt = mix(0.6, 0.08, 1.0 - ecf) * mix(1.0, 0.5, eoe);
        c = mix(c, mix(esp, ew_, ewt), hist_valid);
    }
