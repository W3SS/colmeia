//! F46 — Renderização do grafo em caracteres braille (2×4 dots por caractere).
//!
//! Função pura e testável: posições determinísticas (anel equiespaçado),
//! arestas como linhas de Bresenham, composição dos pontos braille por
//! caractere. Usada na aba "Grafo" da TUI e exportável para logs.

/// Nó do grafo: (id, rótulo). Aresta: (src_id, dst_id, peso).
pub fn render_braille(
    nodes: &[(i64, String)],
    edges: &[(i64, i64, f64)],
    width_chars: usize,
    height_chars: usize,
) -> String {
    let w = width_chars.max(1);
    let h = height_chars.max(1);
    let dot_w = w * 2;
    let dot_h = h * 4;
    let mut dots = vec![vec![false; dot_w]; dot_h];

    // posições em anel (determinísticas)
    let n = nodes.len();
    let mut positions: Vec<(usize, usize)> = Vec::with_capacity(n);
    if n == 1 {
        positions.push((dot_w / 2, dot_h / 2));
    } else if n > 0 {
        let rx = (dot_w as f64 / 2.0) - 1.0;
        let ry = (dot_h as f64 / 2.0) - 1.0;
        let cx = dot_w as f64 / 2.0;
        let cy = dot_h as f64 / 2.0;
        for i in 0..n {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            positions.push((
                (cx + rx * theta.cos()).round().clamp(0.0, dot_w as f64 - 1.0) as usize,
                (cy + ry * theta.sin()).round().clamp(0.0, dot_h as f64 - 1.0) as usize,
            ));
        }
    }

    let pos_of = |id: i64| -> Option<(usize, usize)> {
        nodes
            .iter()
            .position(|(nid, _)| *nid == id)
            .map(|i| positions[i])
    };

    // arestas ordenadas por peso (as mais fortes por cima)
    let mut sorted: Vec<&(i64, i64, f64)> = edges.iter().collect();
    sorted.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    for (a, b, _) in sorted {
        if let (Some(p0), Some(p1)) = (pos_of(*a), pos_of(*b)) {
            for (x, y) in bresenham(p0, p1) {
                if x < dot_w && y < dot_h {
                    dots[y][x] = true;
                }
            }
        }
    }

    // nós por cima das arestas (quadrado 2x2 de pontos)
    for &(x, y) in &positions {
        for dy in 0..2 {
            for dx in 0..2 {
                let px = (x + dx).min(dot_w - 1);
                let py = (y + dy).min(dot_h - 1);
                dots[py][px] = true;
            }
        }
    }

    // compõe caracteres braille
    let mut out = String::with_capacity((w + 1) * h);
    for cy in 0..h {
        for cx in 0..w {
            let mut bits = 0u8;
            for (dx, dy, bit) in BRAILLE_MAP {
                let px = cx * 2 + dx;
                let py = cy * 4 + dy;
                if px < dot_w && py < dot_h && dots[py][px] {
                    bits |= bit;
                }
            }
            out.push(char::from_u32(0x2800 + bits as u32).unwrap_or(' '));
        }
        out.push('\n');
    }
    out
}

// (dx, dy, bit) — mapeamento dos pontos braille na célula 2×4
const BRAILLE_MAP: [(usize, usize, u8); 8] = [
    (0, 0, 0x01), // dot 1
    (0, 1, 0x02), // dot 2
    (0, 2, 0x04), // dot 3
    (1, 0, 0x08), // dot 4
    (1, 1, 0x10), // dot 5
    (1, 2, 0x20), // dot 6
    (0, 3, 0x40), // dot 7
    (1, 3, 0x80), // dot 8
];

fn bresenham(p0: (usize, usize), p1: (usize, usize)) -> Vec<(usize, usize)> {
    let (x0, y0) = (p0.0 as isize, p0.1 as isize);
    let (x1, y1) = (p1.0 as isize, p1.1 as isize);
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        points.push((x as usize, y as usize));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    points
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn dimensoes_corretas() {
        let nodes = vec![(1, "a".into()), (2, "b".into())];
        let out = render_braille(&nodes, &[(1, 2, 1.0)], 20, 6);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 6);
        assert!(lines.iter().all(|l| l.chars().count() == 20));
    }

    #[test]
    fn aresta_marca_pontos() {
        let nodes = vec![(1, "a".into()), (2, "b".into())];
        let with_edge = render_braille(&nodes, &[(1, 2, 1.0)], 20, 6);
        let without = render_braille(&nodes, &[], 20, 6);
        let count = |s: &str| s.chars().filter(|c| *c as u32 >= 0x2801).count();
        assert!(
            count(&with_edge) > count(&without),
            "a aresta deve acender pontos adicionais"
        );
    }

    #[test]
    fn determinismo() {
        let nodes = vec![(1, "a".into()), (2, "b".into()), (3, "c".into())];
        let e = vec![(1, 2, 0.9), (2, 3, 0.5)];
        assert_eq!(render_braille(&nodes, &e, 10, 5), render_braille(&nodes, &e, 10, 5));
    }

    #[test]
    fn vazio_nao_panica() {
        let out = render_braille(&[], &[], 5, 2);
        assert_eq!(out.lines().count(), 2);
    }
}
