/* The WitnessGlass Behavioral Spectroscope.
   Bundled with the example binary. No external asset, no network access beyond
   the one same-origin fetch below, nothing persisted.

   This script DRAWS. It performs no transform, computes no distance, re-buckets
   nothing, and never sees raw NDJSON. Everything it renders was computed in Rust
   before the listener bound — decision:6 — so the only arithmetic here is mapping
   a millisecond onto an x coordinate and a magnitude onto an opacity.

   Recording-derived text reaches the document through textContent and nothing
   else. No API that parses a string into markup is called anywhere in this file,
   and there is no inline handler, so no payload string has a path on which it
   could become an element. The guards in tests/spectroscope.rs assert the
   absence of those APIs by name, which is why none of them is written out even
   in a comment. */

(function () {
    "use strict";

    var SVG_NS = "http://www.w3.org/2000/svg";
    var VIEW_W = 1000;

    var state = {
        doc: null,
        perspective: "overview",
        haarDimension: null,
        profileDimension: null,
        profileWindow: null,
        scaleIndex: 0,
        showAllDimensions: false,
        fadeNull: false,
        highlight: null,
        selectedFinding: null
    };

    /* ---- tiny DOM helpers ------------------------------------------------ */

    function el(tag, className, text) {
        var node = document.createElement(tag);
        if (className) { node.className = className; }
        if (text !== undefined && text !== null) { node.textContent = String(text); }
        return node;
    }

    function shape(tag, attributes) {
        var node = document.createElementNS(SVG_NS, tag);
        Object.keys(attributes).forEach(function (name) {
            node.setAttribute(name, String(attributes[name]));
        });
        return node;
    }

    function canvas(height, ariaLabel) {
        var node = shape("svg", {
            viewBox: "0 0 " + VIEW_W + " " + height,
            preserveAspectRatio: "none",
            role: "img",
            "aria-label": ariaLabel
        });
        node.style.height = height + "px";
        return node;
    }

    function clear(node) {
        while (node.firstChild) { node.removeChild(node.firstChild); }
    }

    /* ---- formatting ------------------------------------------------------ */

    function seconds(ms) {
        if (ms < 1000) { return ms + " ms"; }
        var value = ms / 1000;
        return (value >= 100 ? value.toFixed(0) : value.toFixed(1)).replace(/\.0$/, "") + " s";
    }

    function clock(ms) {
        var total = Math.round(ms / 1000);
        var minutes = Math.floor(total / 60);
        var rest = total % 60;
        return minutes + ":" + (rest < 10 ? "0" : "") + rest;
    }

    function shortLabel(label) {
        var colon = label.indexOf(":");
        return colon === -1 ? label : label.slice(colon + 1);
    }

    function span() { return Math.max(1, state.doc.provenance.span_ms); }
    function toX(ms) { return (ms / span()) * VIEW_W; }

    /* ---- track scaffolding ---------------------------------------------- */

    function track(labelNodes, body, extraClass) {
        var row = el("div", "track" + (extraClass ? " " + extraClass : ""));
        var label = el("div", "track-label");
        labelNodes.forEach(function (node) { label.appendChild(node); });
        var holder = el("div", "track-body");
        holder.appendChild(body);
        row.appendChild(label);
        row.appendChild(holder);
        return row;
    }

    function labelLine(className, text) { return el("span", className, text); }

    function classTag(claimClass) {
        var glyphs = { planted: "■", observed: "▲", interpretation: "●" };
        var node = el("span", "class class-" + claimClass);
        node.textContent = glyphs[claimClass] + " " + claimClass;
        return node;
    }

    function axis() {
        var row = el("div", "axis");
        row.appendChild(el("div", "track-label"));
        var body = el("div", "axis-body");
        var total = span();
        var count = 6;
        for (var index = 0; index <= count; index += 1) {
            var ms = (total / count) * index;
            var tick = el("span", "tick", clock(ms));
            if (index === 0) { tick.className = "tick tick-first"; }
            if (index === count) { tick.className = "tick tick-last"; }
            tick.style.left = ((ms / total) * 100) + "%";
            body.appendChild(tick);
        }
        row.appendChild(body);
        return row;
    }

    /* A single cursor and a single highlight layer over the whole stack, so
       every track shares exactly the same geometry instead of each drawing its
       own and drifting. */
    function decorate(stack) {
        var cursor = el("div", "cursor");
        var readout = el("div", "cursor-readout");
        cursor.appendChild(readout);
        stack.appendChild(cursor);

        var bodies = stack.querySelectorAll(".track-body");
        if (!bodies.length) { return; }

        stack.addEventListener("mousemove", function (event) {
            var reference = bodies[0].getBoundingClientRect();
            var fraction = (event.clientX - reference.left) / reference.width;
            if (fraction < 0 || fraction > 1) { cursor.style.display = "none"; return; }
            cursor.style.display = "block";
            cursor.style.left = (event.clientX - stack.getBoundingClientRect().left) + "px";
            readout.textContent = clock(fraction * span());
        });
        stack.addEventListener("mouseleave", function () { cursor.style.display = "none"; });

        if (state.highlight) {
            state.highlight.forEach(function (range, index) {
                var band = el("div", "cursor");
                band.style.display = "block";
                band.style.background = index === 0 ? "var(--derived)" : "var(--recurrence)";
                band.style.opacity = "0.28";
                var reference = bodies[0];
                var left = reference.offsetLeft + (range[0] / span()) * reference.offsetWidth;
                var width = Math.max(2, ((range[1] - range[0]) / span()) * reference.offsetWidth);
                band.style.left = left + "px";
                band.style.width = width + "px";
                stack.appendChild(band);
            });
        }
    }

    /* ---- ground truth (planted) ----------------------------------------- */

    function groundTruthTrack() {
        var truth = state.doc.ground_truth;
        if (!truth) { return null; }
        var art = canvas(18, "Regions this fixture's generator planted");
        truth.regions.forEach(function (region) {
            var x = toX(region.start_ms);
            var width = Math.max(1, toX(region.end_ms) - x);
            var fill = {
                baseline: "var(--baseline)",
                motif: "var(--motif)",
                regime: "var(--regime)",
                recurrence: "var(--recurrence)"
            }[region.kind];
            var rect = shape("rect", {
                x: x, y: 3, width: width, height: 12, fill: fill,
                opacity: region.kind === "baseline" ? 0.35 : 0.8
            });
            var title = document.createElementNS(SVG_NS, "title");
            title.textContent = region.label + " — " + region.detail;
            rect.appendChild(title);
            art.appendChild(rect);
        });
        return track(
            [classTag("planted"), labelLine("name", "planted structure")],
            art
        );
    }

    /* The planted band's colours mean four different kinds of structure, so the
       key names them. Colour alone would be decoration. */
    function regionKey() {
        var truth = state.doc.ground_truth;
        var wrap = el("div", "region-key");
        if (!truth) { return wrap; }
        var seen = [];
        truth.regions.forEach(function (region) {
            if (seen.indexOf(region.kind) !== -1) { return; }
            seen.push(region.kind);
            var item = el("span");
            var swatch = el("i");
            swatch.style.background = "var(--" + region.kind + ")";
            swatch.style.opacity = region.kind === "baseline" ? "0.35" : "0.8";
            item.appendChild(swatch);
            item.appendChild(document.createTextNode(region.kind));
            wrap.appendChild(item);
        });
        return wrap;
    }

    function groundTruthKey() {
        var truth = state.doc.ground_truth;
        var wrap = el("div", "empty-note");
        if (!truth) {
            wrap.textContent =
                "No ground truth: this recording is not a synthetic fixture, so nothing here " +
                "knows what it contains. Every finding below is a candidate.";
            return wrap;
        }
        wrap.textContent = truth.fixture + " — regions read from " + truth.sourced_from +
            ". Motif period " + seconds(truth.motif_period_ms) +
            ", one instance lasting about " + seconds(truth.motif_instance_ms) + ".";
        return wrap;
    }

    /* ---- behavioural raster (observed) ----------------------------------- */

    function activeDimensions() {
        var dims = state.doc.dimensions;
        return dims.map(function (dimension, index) { return { dimension: dimension, index: index }; })
            .filter(function (entry) {
                return state.showAllDimensions || entry.dimension.occupied > 0;
            });
    }

    function rasterTrack(entry, scale) {
        var row = scale.rows[entry.index] || [];
        var bucket = scale.bucket_ms;
        // Occupancy and peak are read from the row being drawn, not from the base
        // scale: at a coarser aggregation both change, and a label that mixed the
        // two would read "105 of 61 buckets".
        var occupied = 0;
        var peak = 1;
        for (var scan = 0; scan < row.length; scan += 1) {
            if (row[scan]) { occupied += 1; }
            if (row[scan] > peak) { peak = row[scan]; }
        }
        var art = canvas(10, shortLabel(entry.dimension.label) + " activity over time");
        for (var index = 0; index < row.length; index += 1) {
            var value = row[index];
            if (!value) { continue; }
            var width = Math.max(0.7, toX(bucket));
            var x = Math.min(toX(index * bucket), VIEW_W - width);
            // Presence is the bar; magnitude is its weight. A single event and a
            // burst have to be tellable apart without a legend.
            var strength = 0.35 + 0.65 * Math.min(1, value / peak);
            art.appendChild(shape("rect", {
                x: x, y: 1, width: width, height: 8,
                fill: "var(--observed)", opacity: strength.toFixed(3)
            }));
        }
        var label = [
            labelLine("name", shortLabel(entry.dimension.label)),
            labelLine("meta", occupied + " of " + scale.samples + " buckets")
        ];
        var node = track(label, art);
        node.title = entry.dimension.label;
        return node;
    }

    /* ---- Haar (observed) -------------------------------------------------- */

    function haarFor(label) {
        return state.doc.haar.filter(function (view) { return view.label === label; })[0];
    }

    function haarTracks(view) {
        if (!view) { return []; }
        if (view.silence) {
            var note = el("div", "empty-note");
            note.textContent = {
                empty: "This dimension is zero everywhere: nothing varied, at any scale.",
                constant: "This dimension never changes, so no scale carries a contrast.",
                only_in_remainders:
                    "Not a flat dimension. Every non-zero sample fell into an odd-length " +
                    "remainder and reached no scale at all — the transform's limitation, " +
                    "not the recording's."
            }[view.silence];
            return [note];
        }
        return view.levels.map(function (level) {
            var art = canvas(11, "Haar detail at the " + seconds(level.scale_ms) + " scale");
            var count = level.magnitude.length;
            var width = span() / Math.max(1, count);
            for (var index = 0; index < count; index += 1) {
                var magnitude = level.magnitude[index];
                if (!magnitude) { continue; }
                var relative = level.level_max > 0 ? magnitude / level.level_max : 0;
                var drawn = Math.max(0.7, toX(width));
                art.appendChild(shape("rect", {
                    x: Math.min(toX(index * width), VIEW_W - drawn), y: 0,
                    width: drawn, height: 11,
                    fill: "var(--derived)", opacity: (0.12 + 0.88 * relative).toFixed(3)
                }));
            }
            var ratio = level.ratio_to_impulse_null;
            var node = track(
                [labelLine("name", seconds(level.scale_ms)),
                 labelLine("meta", "×" + ratio.toFixed(2) + " vs null")],
                art
            );
            // The null filter is a filter, not a recomputation: rows the isolated
            // impulse already explains are faded so what remains is the surplus.
            if (state.fadeNull && ratio > 0.75 && ratio < 1.25) { node.className += " dimmed"; }
            return node;
        });
    }

    function haarNullPanel(view) {
        var wrap = el("div");
        if (!view || view.silence) { return wrap; }
        var table = el("table", "grid");
        var caption = el("caption",
            null,
            "Detail energy by scale, against the share an isolated event alone would produce. " +
            "A ratio near 1 means the scale is indistinguishable from sparsity.");
        table.appendChild(caption);
        var head = el("tr");
        ["scale", "share", "impulse null", "ratio", ""].forEach(function (title) {
            head.appendChild(el("th", null, title));
        });
        table.appendChild(head);
        view.levels.forEach(function (level) {
            var row = el("tr");
            row.appendChild(el("td", null, seconds(level.scale_ms)));
            row.appendChild(el("td", null, (100 * level.share).toFixed(2) + "%"));
            row.appendChild(el("td", null, (100 * level.impulse_null_share).toFixed(2) + "%"));
            row.appendChild(el("td", null, "×" + level.ratio_to_impulse_null.toFixed(2)));
            var cell = el("td", "bar-cell");
            var bar = el("span", "bar");
            bar.style.width = Math.min(100, level.ratio_to_impulse_null * 25).toFixed(1) + "%";
            cell.appendChild(bar);
            row.appendChild(cell);
            table.appendChild(row);
        });
        wrap.appendChild(table);
        return wrap;
    }

    /* ---- Matrix Profile (observed) --------------------------------------- */

    function profileFor(label) {
        return state.doc.profiles.filter(function (view) { return view.label === label; })[0];
    }

    function windowFor(view, windowMs) {
        if (!view) { return null; }
        var found = view.windows.filter(function (entry) { return entry.window_ms === windowMs; });
        return found.length ? found[0] : view.windows[0];
    }

    function profileTrack(entry) {
        var art = canvas(46, "Matrix Profile distance over time");
        if (!entry) { return track([labelLine("name", "no profile")], art); }
        var values = entry.profile;
        var finite = values.filter(function (value) { return value !== null; });
        var top = finite.length ? Math.max.apply(null, finite) : 1;
        var step = span() / Math.max(1, values.length);
        var commands = [];
        var pen = false;
        for (var index = 0; index < values.length; index += 1) {
            var value = values[index];
            if (value === null) { pen = false; continue; }
            var x = toX(index * step).toFixed(2);
            var y = (44 - (value / Math.max(top, 1e-9)) * 42).toFixed(2);
            commands.push((pen ? "L" : "M") + x + " " + y);
            pen = true;
        }
        if (commands.length) {
            art.appendChild(shape("path", {
                d: commands.join(" "),
                fill: "none",
                stroke: "var(--observed)",
                "stroke-width": 1,
                "vector-effect": "non-scaling-stroke"
            }));
        }
        return track(
            [labelLine("name", "distance"),
             labelLine("meta", seconds(entry.window_ms) + " window"),
             labelLine("meta", "gaps = excluded")],
            art
        );
    }

    function findingNode(kind, headParts, prose, ranges, occupancy) {
        var wrap = el("div", "finding finding-" + kind);
        var head = el("div", "finding-head");
        headParts.forEach(function (part) {
            head.appendChild(el("span", part[0], part[1]));
        });
        wrap.appendChild(head);
        if (occupancy) { wrap.appendChild(occupancy); }
        if (prose) { wrap.appendChild(el("p", null, prose)); }
        if (ranges) {
            var button = el("button", null,
                ranges.length > 1 ? "highlight both spans" : "highlight this span");
            button.type = "button";
            var key = JSON.stringify(ranges);
            button.setAttribute("aria-pressed", state.selectedFinding === key ? "true" : "false");
            button.addEventListener("click", function () {
                if (state.selectedFinding === key) {
                    state.selectedFinding = null;
                    state.highlight = null;
                } else {
                    state.selectedFinding = key;
                    state.highlight = ranges;
                }
                render();
            });
            wrap.appendChild(button);
        }
        return wrap;
    }

    function occupancyNode(match) {
        var node = el("div", "occupancy");
        var a = el("span", null, "window A holds ");
        var strongA = el("strong", null, match.a_occupancy);
        a.appendChild(strongA);
        a.appendChild(document.createTextNode(" non-empty bucket" + (match.a_occupancy === 1 ? "" : "s")));
        var b = el("span", null, "window B holds ");
        var strongB = el("strong", null, match.b_occupancy);
        b.appendChild(strongB);
        b.appendChild(document.createTextNode(" non-empty bucket" + (match.b_occupancy === 1 ? "" : "s")));
        node.appendChild(a);
        node.appendChild(b);
        return node;
    }

    function profileFindings(entry) {
        var wrap = el("div");
        if (!entry) { return wrap; }

        var explained = false;
        entry.matches.slice(0, 3).forEach(function (match, rank) {
            var ranges = [[match.a_start_ms, match.a_end_ms], [match.b_start_ms, match.b_end_ms]];
            var prose;
            if (!match.trivial) {
                prose = "Both windows carry enough activity for the shape, rather than the " +
                        "position of a single event, to be doing the matching.";
            } else if (!explained) {
                explained = true;
                prose = "Both windows hold one or two non-empty buckets at the same offset. " +
                        "After subsequence normalization they are mathematically identical, so a " +
                        "distance of zero was arithmetic rather than evidence. This does not " +
                        "establish that similar behaviour recurred.";
            } else {
                prose = "Trivial for the same reason as above: lone events at a shared offset.";
            }
            wrap.appendChild(findingNode(
                match.trivial ? "trivial" : "match",
                [["tag", match.trivial ? "candidate · trivial" : "candidate match"],
                 ["spans", clock(match.a_start_ms) + "–" + clock(match.a_end_ms) +
                     "  ↔  " + clock(match.b_start_ms) + "–" + clock(match.b_end_ms)],
                 ["metric", "rank " + (rank + 1) + " · distance " + match.distance.toFixed(4)]],
                prose,
                ranges,
                occupancyNode(match)
            ));
        });

        if (entry.discord) {
            wrap.appendChild(findingNode(
                "discord",
                [["tag", "candidate discord"],
                 ["spans", clock(entry.discord[0]) + "–" + clock(entry.discord[1])],
                 ["metric", "distance " + entry.discord[2].toFixed(4)]],
                "The span least like anything else in this dimension, at this window, in this " +
                "recording. In a mostly-empty signal that is a better-posed question than " +
                "similarity, because dense stretches are rare and empty ones are everywhere.",
                [[entry.discord[0], entry.discord[1]]],
                null
            ));
        }

        var truth = state.doc.ground_truth;
        if (truth) {
            var motif = truth.regions.filter(function (r) { return r.kind === "motif"; })[0];
            var recurrence = truth.regions.filter(function (r) { return r.kind === "recurrence"; })[0];
            if (motif && recurrence) {
                wrap.appendChild(findingNode(
                    "match",
                    [["tag", "■ planted · for comparison"],
                     ["spans", clock(motif.start_ms) + "–" + clock(motif.end_ms) +
                         "  ↔  " + clock(recurrence.start_ms) + "–" + clock(recurrence.end_ms)],
                     ["metric", "the figure this fixture was built to contain"]],
                    "Not a detector output. This is where the generator put the repeated figure and " +
                    "its recurrence. Highlight it and compare against what the profile ranked first.",
                    [[motif.start_ms, motif.end_ms], [recurrence.start_ms, recurrence.end_ms]],
                    null
                ));
            }
        }
        return wrap;
    }

    function profileNullTable(view) {
        var wrap = el("div");
        if (!view) { return wrap; }
        var table = el("table", "grid");
        table.appendChild(el("caption", null,
            "Every window of the preregistered ladder for this dimension. The null is the same " +
            "values in a fixed-seed shuffle: same density, no temporal order. A low distance is " +
            "not interesting when the null reaches it too."));
        var head = el("tr");
        ["window", "best", "null", "separation", "constant", ""].forEach(function (title) {
            head.appendChild(el("th", null, title));
        });
        table.appendChild(head);
        view.windows.forEach(function (entry) {
            var row = el("tr");
            var best = entry.matches.length ? entry.matches[0].distance : null;
            row.appendChild(el("td", null, seconds(entry.window_ms)));
            row.appendChild(el("td", null, best === null ? "—" : best.toFixed(3)));
            row.appendChild(el("td", null,
                entry.null_best_distance === null ? "—" : entry.null_best_distance.toFixed(3)));
            var separation = el("td", null,
                entry.separation === null ? "—" :
                    (entry.separation >= 0 ? "+" : "") + entry.separation.toFixed(3));
            if (entry.separation !== null && Math.abs(entry.separation) < 1e-6) {
                separation.className = "zero";
                separation.textContent = "+0.000 — none";
            }
            row.appendChild(separation);
            row.appendChild(el("td", null, (100 * entry.constant_fraction).toFixed(0) + "%"));
            var cell = el("td", "bar-cell");
            var bar = el("span", "bar");
            bar.style.width = Math.max(0, Math.min(100, (entry.separation || 0) * 160)).toFixed(1) + "%";
            cell.appendChild(bar);
            row.appendChild(cell);
            table.appendChild(row);
        });
        wrap.appendChild(table);
        return wrap;
    }

    /* ---- narrative -------------------------------------------------------- */

    function narrative() {
        var wrap = el("div");
        var glyphs = { planted: "■", observed: "▲", interpretation: "●" };
        state.doc.narrative.forEach(function (step) {
            var node = el("div", "step step-" + step.class);
            node.appendChild(el("span", "step-glyph class-" + step.class, glyphs[step.class]));
            node.appendChild(el("h3", null, step.heading));
            node.appendChild(el("span", "step-class", step.class));
            node.appendChild(el("p", null, step.body));
            wrap.appendChild(node);
        });
        return wrap;
    }

    /* ---- rendering -------------------------------------------------------- */

    function sectionRule(title, note) {
        var row = el("div", "section-rule");
        row.appendChild(el("span", "section-title", title));
        if (note) { row.appendChild(el("span", "section-note", note)); }
        return row;
    }

    function overviewStack() {
        var stack = el("div", "track-stack");
        stack.appendChild(axis());
        var planted = groundTruthTrack();
        if (planted) { stack.appendChild(planted); }

        stack.appendChild(sectionRule("what happened",
            "observed activity, " + state.doc.provenance.base_bucket_ms + " ms buckets"));
        var scale = state.doc.scales[0];
        var top = activeDimensions().slice(0, 6);
        top.forEach(function (entry) { stack.appendChild(rasterTrack(entry, scale)); });

        var haar = haarFor(state.haarDimension);
        stack.appendChild(sectionRule("Haar, across scales",
            shortLabel(state.haarDimension) + " — rows are window widths, not frequencies"));
        haarTracks(haar).slice(0, 5).forEach(function (node) { stack.appendChild(node); });

        var profile = profileFor(state.profileDimension);
        stack.appendChild(sectionRule("Matrix Profile, one window",
            shortLabel(state.profileDimension) + " — distance to the nearest other span"));
        stack.appendChild(profileTrack(windowFor(profile, state.profileWindow)));
        stack.appendChild(axis());
        return stack;
    }

    function render() {
        var doc = state.doc;
        if (!doc) { return; }

        document.getElementById("loading").hidden = true;

        // provenance strip
        var provenance = document.getElementById("provenance");
        clear(provenance);
        var empties = doc.provenance.empty_samples;
        var pct = (100 * empties / Math.max(1, doc.provenance.samples)).toFixed(1);
        [["session", doc.provenance.session_id || "none"],
         ["records", String(doc.provenance.records)],
         ["schema", doc.provenance.schema_version ? "v" + doc.provenance.schema_version : "none"],
         ["base sampling", doc.provenance.base_bucket_ms + " ms"],
         ["samples", String(doc.provenance.samples)],
         ["empty", pct + "%"],
         ["span", clock(doc.provenance.span_ms)],
         ["axis", "recorded_at — descriptive, not the canonical order"]
        ].forEach(function (pair) {
            var group = el("div");
            group.appendChild(el("dt", null, pair[0]));
            group.appendChild(el("dd", null, pair[1]));
            provenance.appendChild(group);
        });
        if (doc.provenance.truncated) {
            var group = el("div");
            group.appendChild(el("dt", null, "scope"));
            var value = el("dd", "warn", "valid prefix only — recording stops mid-record");
            group.appendChild(value);
            provenance.appendChild(group);
        }

        // overview
        var overview = document.getElementById("overview-stack");
        clear(overview);
        var overviewInner = overviewStack();
        overview.appendChild(overviewInner);
        decorate(overviewInner);
        overview.appendChild(regionKey());
        var narrativeHolder = document.getElementById("narrative");
        clear(narrativeHolder);
        narrativeHolder.appendChild(groundTruthKey());
        narrativeHolder.appendChild(narrative());

        // haar
        var haarStack = document.getElementById("haar-stack");
        clear(haarStack);
        var haarInner = el("div", "track-stack");
        haarInner.appendChild(axis());
        var plantedHaar = groundTruthTrack();
        if (plantedHaar) { haarInner.appendChild(plantedHaar); }
        var haarView = haarFor(state.haarDimension);
        haarTracks(haarView).forEach(function (node) { haarInner.appendChild(node); });
        haarInner.appendChild(axis());
        haarStack.appendChild(haarInner);
        decorate(haarInner);
        haarStack.appendChild(regionKey());
        var nullPanel = document.getElementById("haar-null-panel");
        clear(nullPanel);
        nullPanel.appendChild(haarNullPanel(haarView));

        // matrix profile
        var profileStack = document.getElementById("profile-stack");
        clear(profileStack);
        var profileInner = el("div", "track-stack");
        profileInner.appendChild(axis());
        var plantedProfile = groundTruthTrack();
        if (plantedProfile) { profileInner.appendChild(plantedProfile); }
        var profileView = profileFor(state.profileDimension);
        var chosen = windowFor(profileView, state.profileWindow);
        var entryForRaster = state.doc.dimensions
            .map(function (dimension, index) { return { dimension: dimension, index: index }; })
            .filter(function (item) { return item.dimension.label === state.profileDimension; })[0];
        if (entryForRaster) {
            profileInner.appendChild(rasterTrack(entryForRaster, state.doc.scales[0]));
        }
        profileInner.appendChild(profileTrack(chosen));
        profileInner.appendChild(axis());
        profileStack.appendChild(profileInner);
        decorate(profileInner);
        profileStack.appendChild(regionKey());

        var findings = document.getElementById("profile-findings");
        clear(findings);
        findings.appendChild(profileFindings(chosen));
        var nullTable = document.getElementById("profile-null");
        clear(nullTable);
        nullTable.appendChild(profileNullTable(profileView));

        renderLadder(profileView);

        // raw
        var rawStack = document.getElementById("raw-stack");
        clear(rawStack);
        var rawInner = el("div", "track-stack");
        rawInner.appendChild(axis());
        var plantedRaw = groundTruthTrack();
        if (plantedRaw) { rawInner.appendChild(plantedRaw); }
        var scale = doc.scales[state.scaleIndex];
        activeDimensions().forEach(function (entry) {
            rawInner.appendChild(rasterTrack(entry, scale));
        });
        rawInner.appendChild(axis());
        rawStack.appendChild(rawInner);
        decorate(rawInner);
        rawStack.appendChild(regionKey());
        document.getElementById("scale-readout").textContent =
            seconds(scale.bucket_ms) + " buckets · " + scale.samples + " of them";
    }

    function renderLadder(view) {
        var holder = document.getElementById("profile-ladder");
        clear(holder);
        var windows = view ? view.windows.map(function (entry) { return entry.window_ms; })
                           : state.doc.ladder_ms;
        windows.forEach(function (windowMs) {
            var button = el("button", null, seconds(windowMs));
            button.type = "button";
            button.setAttribute("aria-pressed", windowMs === state.profileWindow ? "true" : "false");
            button.addEventListener("click", function () {
                state.profileWindow = windowMs;
                state.highlight = null;
                state.selectedFinding = null;
                render();
            });
            holder.appendChild(button);
        });
    }

    /* ---- controls --------------------------------------------------------- */

    function fillSelect(id, labels, selected, onChange) {
        var node = document.getElementById(id);
        clear(node);
        labels.forEach(function (label) {
            var option = document.createElement("option");
            option.value = label;
            option.textContent = label;
            if (label === selected) { option.selected = true; }
            node.appendChild(option);
        });
        node.addEventListener("change", function () { onChange(node.value); });
    }

    function wirePerspectives() {
        var buttons = Array.prototype.slice.call(
            document.querySelectorAll('.perspectives [role="tab"]'));
        function select(name) {
            state.perspective = name;
            buttons.forEach(function (button) {
                var active = button.dataset.perspective === name;
                button.setAttribute("aria-selected", active ? "true" : "false");
                button.tabIndex = active ? 0 : -1;
                document.getElementById("panel-" + button.dataset.perspective).hidden = !active;
            });
        }
        buttons.forEach(function (button) {
            button.addEventListener("click", function () { select(button.dataset.perspective); });
            button.addEventListener("keydown", function (event) {
                var index = buttons.indexOf(button);
                var next = null;
                if (event.key === "ArrowRight") { next = buttons[(index + 1) % buttons.length]; }
                if (event.key === "ArrowLeft") {
                    next = buttons[(index - 1 + buttons.length) % buttons.length];
                }
                if (next) {
                    event.preventDefault();
                    select(next.dataset.perspective);
                    next.focus();
                }
            });
        });
        select("overview");
    }

    function boot(doc) {
        state.doc = doc;
        var haarLabels = doc.haar.map(function (view) { return view.label; });
        var profileLabels = doc.profiles.map(function (view) { return view.label; });

        // Default to the dimension the fixture says carries the motif, when there
        // is a fixture; otherwise to the busiest profiled one.
        var preferred = doc.ground_truth ? doc.ground_truth.motif_only_dimension : null;
        state.haarDimension = haarLabels.indexOf(preferred) !== -1 ? preferred : haarLabels[0];
        state.profileDimension =
            profileLabels.indexOf(preferred) !== -1 ? preferred : profileLabels[0];
        state.profileWindow = doc.ladder_ms.length > 2 ? doc.ladder_ms[2] : doc.ladder_ms[0];

        fillSelect("haar-dimension", haarLabels, state.haarDimension, function (value) {
            state.haarDimension = value;
            render();
        });
        fillSelect("profile-dimension", profileLabels, state.profileDimension, function (value) {
            state.profileDimension = value;
            state.highlight = null;
            state.selectedFinding = null;
            render();
        });

        var scrubber = document.getElementById("scale-scrubber");
        scrubber.max = String(doc.scales.length - 1);
        scrubber.addEventListener("input", function () {
            state.scaleIndex = Number(scrubber.value);
            render();
        });

        document.getElementById("haar-null").addEventListener("change", function (event) {
            state.fadeNull = event.target.checked;
            render();
        });
        document.getElementById("show-all-dimensions").addEventListener("change", function (event) {
            state.showAllDimensions = event.target.checked;
            render();
        });

        wirePerspectives();
        render();
        window.addEventListener("resize", render);
    }

    fetch("/spectroscope.json?c={{CAPABILITY}}", { cache: "no-store" })
        .then(function (response) {
            if (!response.ok) { throw new Error("the derived document could not be read"); }
            return response.json();
        })
        .then(boot)
        .catch(function (error) {
            var node = document.getElementById("loading");
            node.textContent = "Could not load the derived document: " + error.message;
            node.className = "failure";
        });
}());
