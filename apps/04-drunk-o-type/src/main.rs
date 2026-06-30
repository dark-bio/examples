use std::error::Error;
use std::fs;
use std::path::Path;

// ── SNP panel ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Flush,
    Reward,
    Histamine,
}

#[derive(Debug, Clone, Copy)]
struct SnpTarget {
    rsid: &'static str,
    gene: &'static str,
    // risk_allele is the LITERAL nucleotide letter (A/C/G/T on forward strand)
    // associated with the phenotype this SNP contributes to. NOT a synonym for
    // ALT — for rs1229984 the GRCh38 reference happens to encode the rare/risk
    // allele. Always derive from the biology, never assume "ALT = risk". It is
    // compared against the alleles the `rsids/<rs#>/genotype` lens reports.
    risk_allele: &'static str,
    axis: Axis,
    short: &'static str,
    blurb: &'static str,
}

// Positions + bases per dbSNP / Ensembl GRCh38 forward-strand convention.
// risk_allele is the LITERAL nucleotide associated with the phenotype — derived
// from the biology (which amino acid / functional change is the "risk" one),
// not from REF/ALT status.
const TARGETS: &[SnpTarget] = &[
    // ── Flush panel: how much acetaldehyde piles up when you drink? ──
    SnpTarget {
        rsid: "rs671",
        gene: "ALDH2",
        risk_allele: "A", // ALDH2*2 (Lys487, broken enzyme) = ALT
        axis: Axis::Flush,
        short: "broken acetaldehyde clearance",
        blurb: "Glu487Lys in ALDH2. Dominant-negative — one broken subunit poisons the whole tetramer (~6% activity). The classic alcohol-flush variant.",
    },
    SnpTarget {
        rsid: "rs1229984",
        gene: "ADH1B",
        // ⚠️ Risk = REF here. GRCh38 happens to encode the rare/derived His48
        // (fast, ADH1B*2) allele at this position. The major worldwide allele
        // (Arg48 / slow / ADH1B*1) is the ALT (C). Most populations are C/C
        // homozygous variant = wild-type slow.
        risk_allele: "T",
        axis: Axis::Flush,
        short: "fast alcohol→acetaldehyde (ADH1B*2)",
        blurb: "Arg48His in ADH1B. The His48 variant (ADH1B*2) oxidizes ethanol 70–100× faster, piling up acetaldehyde. Common in East Asia (~25–70% allele freq), Ashkenazi Jews, parts of the Middle East. Note: GRCh38 encodes His48 (T) at this position despite it being the global minor allele — for most populations the reference genome is the variant, so the 'risk' allele here is REF, not ALT.",
    },
    SnpTarget {
        rsid: "rs2066702",
        gene: "ADH1B",
        risk_allele: "A", // ADH1B*3 (Cys370, fast) = ALT
        axis: Axis::Flush,
        short: "fast alcohol→acetaldehyde (ADH1B*3)",
        blurb: "Arg370Cys in ADH1B. Same fast-metabolizer phenotype as *2, different mutation, common in West/Central African ancestry.",
    },
    // ── Reward panel: does your reward circuitry enjoy alcohol? ──
    SnpTarget {
        rsid: "rs1799971",
        gene: "OPRM1",
        risk_allele: "G", // Asp40 (stronger reward) = ALT
        axis: Axis::Reward,
        short: "stronger alcohol reward",
        blurb: "A118G (Asn40Asp) in OPRM1. The G allele binds β-endorphin tighter — alcohol's reward signal lands harder. Predicts naltrexone response in dependence treatment.",
    },
    SnpTarget {
        rsid: "rs1800497",
        gene: "ANKK1/DRD2",
        risk_allele: "A", // A1 (Lys713, reduced D2 density) = ALT
        axis: Axis::Reward,
        short: "more reward-seeking (Taq1A)",
        blurb: "Glu713Lys in ANKK1, near DRD2. The A (A1) allele tracks with reduced striatal D2 density and stronger reward chasing.",
    },
    SnpTarget {
        rsid: "rs279858",
        gene: "GABRA2",
        // C is the most commonly cited dependence-risk allele per Edenberg
        // 2004 + follow-ups, but direction is genuinely contested in the
        // literature. Treat as "may indicate" rather than confirmed risk.
        risk_allele: "C",
        axis: Axis::Reward,
        short: "alcohol's chill effect (effect direction debated)",
        blurb: "GABA-A α2 subunit. Tracks with alcohol's anxiolytic 'unwind' effect and dependence risk. The risk-allele direction is replicated unevenly across studies — treat the contribution to the score as suggestive, not definitive.",
    },
    // ── Junk panel (Histamine sub-panel): does fermented-drink load wreck you? ──
    // Histamine pathway only: 4× DAO (gut/blood histamine clearance) +
    // 1× HNMT (alternative CNS/airway pathway). Sulfite sensitivity and
    // tyramine tolerance don't have well-replicated common SNPs at the
    // genotype-array level — SUOX has only severe-disease pathogenic variants,
    // and the canonical MAOA tyramine signal is the uVNTR, not a SNP.
    SnpTarget {
        rsid: "rs10156191",
        gene: "AOC1",
        risk_allele: "T", // Met16 (reduced DAO activity) = ALT
        axis: Axis::Histamine,
        short: "DAO Thr16Met (may slow histamine clearance)",
        blurb: "Thr16Met in AOC1/DAO. Associated with reduced histamine-degrading enzyme activity. ~22% MAF in Europeans — one of the most-studied DAO variants.",
    },
    SnpTarget {
        rsid: "rs1049742",
        gene: "AOC1",
        risk_allele: "T", // Phe332 (reduced DAO activity) = ALT
        axis: Axis::Histamine,
        short: "DAO Ser332Phe (may slow histamine clearance)",
        blurb: "Ser332Phe in AOC1/DAO. Associated with altered enzyme kinetics. Less common (~4% MAF) but functionally meaningful in carriers.",
    },
    SnpTarget {
        rsid: "rs1049793",
        gene: "AOC1",
        risk_allele: "G", // Asp645 (reduced DAO activity per most reports) = ALT
        axis: Axis::Histamine,
        short: "DAO His645Asp (may slow histamine clearance)",
        blurb: "His645Asp in AOC1/DAO. The most common DAO variant in Caucasians (~38% MAF). Some studies link reduced enzyme activity to histamine-intolerance symptoms; the effect size is modest and replication is mixed.",
    },
    SnpTarget {
        rsid: "rs2052129",
        gene: "AOC1",
        risk_allele: "T", // Reduced transcription = ALT
        axis: Axis::Histamine,
        short: "DAO promoter (may reduce DAO transcription)",
        blurb: "Promoter variant 2kb upstream of AOC1. Reportedly reduces transcription — fewer DAO enzymes made. ~21% MAF.",
    },
    SnpTarget {
        rsid: "rs11558538",
        gene: "HNMT",
        risk_allele: "T", // Ile105 (reduced HNMT activity) = ALT
        axis: Axis::Histamine,
        short: "HNMT Thr105Ile (may slow CNS/airway histamine clearance)",
        blurb: "Thr105Ile in HNMT. The alternative histamine-clearance pathway (mainly CNS/airways). The Ile105 variant has reduced activity. Linked to asthma, allergic rhinitis, atopic eczema in epidemiology.",
    },
];

// ── Resolved call ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct GenotypeCall {
    alleles: Vec<String>,
    risk_copies: u32,
    // confident = false if any allele was missing (`.`). Uncertain calls are
    // excluded from scoring — they neither contribute risk nor count as
    // "observed" coverage.
    confident: bool,
    // Coordinate and reference base from the rsids lens, for the details table.
    chromosome: String,
    position: String,
    reference: String,
}

// ── Tier classification ────────────────────────────────────────────────────
//
// Flush axis is *not* a simple linear sum — ALDH2 is dominant-negative
// (the enzyme works as a tetramer, one bad subunit kills it). So we tier on
// (ALDH2 risk-allele count, total ADH1B risk-allele count) instead.
//
// Like axis is straight-up additive across the three reward-pathway SNPs.

// Returns (tier_idx, label, emoji). idx is None when ALDH2 is missing — the
// tier is "Inconclusive" and the combo callout / share-worthy headline can't
// fire reliably without the dominant variant.
fn flush_tier(s: &FlushScore) -> (Option<usize>, &'static str, &'static str) {
    if !s.aldh2_observed {
        return (None, "Inconclusive", "❓");
    }
    if s.aldh2_risk >= 2 {
        return (Some(4), "Tomato Mode", "🍅");
    }
    if s.aldh2_risk == 1 {
        return (Some(3), "Glow Stick", "🥵");
    }
    match s.adh1b_risk {
        0 => (Some(0), "Iron Liver", "🍺"),
        1..=2 => (Some(1), "Tank", "🍻"),
        _ => (Some(2), "Lightweight", "🥴"),
    }
}

fn flush_blurb(s: &FlushScore) -> &'static str {
    if !s.aldh2_observed {
        return "rs671 (ALDH2) was not found in your VCF. ALDH2 is the dominant \
                Flush variant — without confirming it, we can't tell whether \
                you're Iron Liver or Tomato Mode. Re-genotype with a panel that \
                covers chr12:111,803,962 (GRCh38) before trusting a Flush call.";
    }
    if s.aldh2_risk >= 2 {
        return "Homozygous broken ALDH2. Functional enzyme: dead. Acetaldehyde — \
                the poison your body makes from alcohol — has nowhere to go. Sip → \
                flush → headache → 'wait why am I sweating?'. Not a lightweight. \
                Pharmacology.";
    }
    if s.aldh2_risk == 1 {
        return "Heterozygous ALDH2. One broken copy poisons the whole tetramer \
                (~6% activity). One drink in, you're glowing. Two drinks in, \
                headache. Real flush, real fast.";
    }
    match s.adh1b_risk {
        0 => "Both ALDH2 copies clear acetaldehyde clean. ADH1B running at \
              standard speed. Hangovers are a choice, not a consequence.",
        1..=2 => "Standard human metabolism. You can keep up with the table \
                  without lighting up. Your liver does not need your help.",
        _ => "ADH1B in turbo mode — your body produces acetaldehyde faster than \
              baseline. Pink cheeks by drink two, mild glow afterwards.",
    }
}

fn like_tier(score: u32) -> (usize, &'static str, &'static str) {
    match score {
        0 => (0, "Stone Cold", "🪨"),
        1 => (1, "Take It Or Leave It", "😐"),
        2..=3 => (2, "Sociable Sipper", "🍷"),
        4..=5 => (3, "Hedonist", "😎"),
        _ => (4, "Wired For It", "🎰"),
    }
}

fn like_blurb(score: u32) -> &'static str {
    match score {
        0 => "Reward circuitry shrugs at alcohol. The buzz doesn't land, the \
              chill doesn't chill. Booze is just bitter water with vibes.",
        1 => "Alcohol's fine. Sometimes nice. Not a thing your brain would \
              chase out of a quiet evening.",
        2..=3 => "Buzz lands. Evening's nicer with a glass. Standard \
                  relationship with booze — neither indifferent nor wired.",
        4..=5 => "Your brain LIKES alcohol. Mu-opioid + dopamine systems both \
                  saying yes. Two glasses is genuinely fun.",
        _ => "Every reward gene voting yes. The buzz is good, the reward chase \
              is strong, the chill is real. Worth knowing about yourself.",
    }
}

fn junk_tier(score: u32) -> (&'static str, &'static str) {
    match score {
        0 => ("Cast Iron", "🛡️"),
        1..=2 => ("Mostly Fine", "🌤️"),
        3..=4 => ("Sniffly Glass", "🤧"),
        5..=7 => ("Wine Headache", "🤕"),
        _ => ("DAO Disaster", "🚒"),
    }
}

fn junk_blurb(score: u32) -> &'static str {
    match score {
        0 => "No flagged DAO or HNMT variants. This panel doesn't suggest \
              histamine-clearance issues — but the panel only tests histamine \
              pathways. Sulfites, tannins, sugar, dehydration, and sleep are \
              not captured here.",
        1..=2 => "One or two flagged variants. May lean toward slower histamine \
                  clearance from fermented drinks; effects are typically mild \
                  and the most common European baseline.",
        3..=4 => "Several DAO variants flagged. The panel leans toward slower \
                  histamine clearance — fermented drinks (red wine especially, \
                  aged beers, kombucha) and aged foods (cheese, cured meats) \
                  are *more likely* to cause symptoms. Worth tracking your own \
                  reactions.",
        5..=7 => "Multiple DAO/HNMT variants flagged. The panel suggests reduced \
                  histamine-clearance capacity. People with this load often \
                  report flushing, headache, congestion, racing heart, or \
                  sneezing after fermented drinks — but symptoms vary widely \
                  and the SNPs are not deterministic.",
        _ => "All or nearly all panel variants flagged. The genotype suggests a \
              substantial histamine-clearance burden, but real-world symptoms \
              depend on diet, gut health, hormones, and many factors not in \
              this panel. If fermented-drink reactions are bothering you, talk \
              to a doctor — DAO supplements help some people, but this is not \
              a diagnosis.",
    }
}

// Indexed by [flush_tier_idx][like_tier_idx]. Tiers in order:
//   Flush: 0=Iron Liver, 1=Tank, 2=Lightweight, 3=Glow Stick, 4=Tomato Mode
//   Like:  0=Stone Cold, 1=Take It Or Leave It, 2=Sociable Sipper, 3=Hedonist, 4=Wired For It
const COMBO_CALLOUTS: [[&str; 5]; 5] = [
    // ── Iron Liver row ──
    [
        "**The Waste.** You could drink anything and you don't even want to. \
         Liver wasted on you (literally never).",
        "**The Designated Driver.** Iron liver, indifferent brain. Always reliable, \
         never the problem. Your friends owe you sushi.",
        "**The Social Lubricator.** Drinks for the room, not the buzz. Built to drink, \
         mildly enjoys it. The platonic ideal of a wedding guest.",
        "**The Connoisseur.** No flush, no fade, real reward. You taste wine instead \
         of inhaling it. Built for slow Tuesdays.",
        "**The High-Functioning Enthusiast.** Built to drink, brain says drink, no \
         flush no slowdown. Probably knows every wine bar in town.",
    ],
    // ── Tank row ──
    [
        "**The Polite Pretender.** Sips politely, never really cares. The drink is a \
         prop, the room is the point.",
        "**The Median Human.** Standard metabolism, take-or-leave reward. The genetic \
         baseline of 'one's enough'.",
        "**The Casual Drinker.** Drinks land, evenings improve, hangovers fair. Nothing \
         alarming, nothing impressive. Functional booze.",
        "**The Happy Drunk.** Brain says yes, body keeps up. Best mood at the bar. \
         Three drinks deep, philosophical. Four drinks, calling exes — careful.",
        "**The Pro.** Reward maxed, metabolism standard. You drink for the buzz and \
         your liver does not file complaints. Watch the cab fare.",
    ],
    // ── Lightweight row ──
    [
        "**The Practical Drinker.** Drinks because it's there, never because they're \
         chasing it. Two beers and you're done — physically and motivationally.",
        "**The Mild Glow.** Pink cheeks by drink two, no chase from the brain. One is \
         enough and you'll know it.",
        "**The Quick Tipsy.** Pink cheeks by drink two, evening's pleasant by drink \
         three. You peak early and gracefully.",
        "**The Lit-Up Hedonist.** Brain loves it, body announces it. Glowing within an \
         hour, content all night. Hide the candid photos.",
        "**The Cheap Date With A Hobby.** Three drinks in, philosophical. Four drinks \
         in, horizontal. Five drinks in, calling someone you shouldn't.",
    ],
    // ── Glow Stick row ──
    [
        "**The Easy Decline.** Body says ouch, brain says meh. The version of broken \
         ALDH2 most likely to skip drinking and not feel deprived.",
        "**The Cautious Glower.** One drink, glowing. Two drinks, pink everywhere. \
         Brain mildly enjoys it, body files complaints. Easy to convince to stop.",
        "**The Reluctant Glower.** Likes the social, hates the flush. Probably drinks \
         anyway and pretends not to notice the sunburn.",
        "**The Internal Conflict.** Reward circuit votes yes, metabolism votes hell no. \
         You'll enjoy the first sip and regret the second.",
        "**The Compulsive Glower.** Body screams stop, brain screams more. The \
         dependence-vulnerable combo for non-Tomato genotypes — worth knowing.",
    ],
    // ── Tomato Mode row ──
    [
        "**The Genetic Teetotaler.** Body says no, brain says meh. The universe is \
         sparing you.",
        "**The Easy Quitter.** One sip and you're a stop sign, and your brain doesn't \
         even care. The luckiest version of the broken-enzyme genotype.",
        "**The Reluctant Tomato.** Pleasant buzz, terrible price. Most Tomatoes here \
         learn to politely decline.",
        "**The Tragic Combination.** Your brain wants what your liver can't process. \
         Real talk: ALDH2-deficient people who drink heavily have *significantly* \
         elevated esophageal cancer risk — well-replicated public health finding, \
         not internet doomscroll. Worth knowing.",
        "**The Tragic Combination, Maxed.** Brain demands what liver can't process. \
         ALDH2-deficient people who drink heavily have *significantly* elevated \
         esophageal cancer risk. Drinking through the flush is the dangerous pathway. \
         Worth knowing.",
    ],
];

fn combo_callout(flush_idx: usize, like_idx: usize) -> &'static str {
    COMBO_CALLOUTS[flush_idx][like_idx]
}

// ── Output sections ────────────────────────────────────────────────────────

fn print_app_header() {
    println!("## 🍻 Drunk-o-type");
    println!();
    println!(
        "Eleven SNPs across three axes: **Flush** (does alcohol make you sick?), \
         **Like** (does your brain enjoy it?), and **Junk** (do drink additives — \
         histamine, tannins, the fermented-stuff load — wreck you?). Each axis \
         bins into 5 tiers."
    );
    println!();
}

fn print_sample_report(results: &[(&'static SnpTarget, Option<GenotypeCall>)]) {
    let (flush_score, reward_score, histamine_score) = score_sample(results);
    let (flush_idx_opt, flush_label, flush_emoji) = flush_tier(&flush_score);
    let (like_idx, like_label, like_emoji) = like_tier(reward_score.risk);
    let (junk_label, junk_emoji) = junk_tier(histamine_score.risk);

    println!(
        "### Your drunk-o-type: {} {} + {} {} + {} {}",
        flush_emoji, flush_label, like_emoji, like_label, junk_emoji, junk_label
    );
    println!();

    // Combo callout only fires if all three axes are confidently classifiable.
    // Flush requires ALDH2 to be observed; otherwise the headline already says
    // Inconclusive and we shouldn't compound that with a confident-sounding
    // 5×5 callout.
    if let Some(flush_idx) = flush_idx_opt {
        println!("> {}", combo_callout(flush_idx, like_idx));
        println!();
    } else {
        println!(
            "> ⚠️ Flush axis is inconclusive — rs671 (ALDH2) is missing from \
             your calls. The combo readout below is omitted because ALDH2 \
             dominates the Flush phenotype and we can't pretend to call it \
             without that marker."
        );
        println!();
    }

    // Flush axis breakdown
    println!("#### 🍻 Flush axis — {} {}", flush_label, flush_emoji);
    println!();
    let aldh2_marker = if flush_score.aldh2_observed {
        "✓ found"
    } else {
        "⚠️ missing"
    };
    println!(
        "ALDH2 risk alleles: **{}/2** ({}) · ADH1B fast alleles: **{}/4** ({}/{} markers)",
        flush_score.aldh2_risk,
        aldh2_marker,
        flush_score.adh1b_risk,
        flush_score.adh1b_observed,
        flush_score.adh1b_expected,
    );
    println!();
    println!("{}", flush_blurb(&flush_score));
    println!();
    print_axis_table(results,Axis::Flush);

    // Like axis breakdown
    println!(
        "#### 🧠 Like axis — {} {} ({}/6 risk · {}/{} markers{})",
        like_label,
        like_emoji,
        reward_score.risk,
        reward_score.observed,
        reward_score.expected,
        if reward_score.fully_covered() {
            ""
        } else {
            " ⚠️"
        },
    );
    println!();
    if !reward_score.fully_covered() {
        println!(
            "*Partial coverage: {} of {} reward markers were called \
             confidently. Tier may be biased toward Stone Cold.*",
            reward_score.observed, reward_score.expected
        );
        println!();
    }
    println!("{}", like_blurb(reward_score.risk));
    println!();
    print_axis_table(results,Axis::Reward);

    // Junk axis breakdown
    println!(
        "#### 🤧 Junk axis — fermented-drink sensitivity — {} {} ({}/10 risk · {}/{} markers{})",
        junk_label,
        junk_emoji,
        histamine_score.risk,
        histamine_score.observed,
        histamine_score.expected,
        if histamine_score.fully_covered() {
            ""
        } else {
            " ⚠️"
        },
    );
    println!();
    if !histamine_score.fully_covered() {
        println!(
            "*Partial coverage: {} of {} histamine markers were called \
             confidently. Tier may be biased toward Cast Iron.*",
            histamine_score.observed, histamine_score.expected
        );
        println!();
    }
    println!("{}", junk_blurb(histamine_score.risk));
    println!();
    print_axis_table(results,Axis::Histamine);
}

#[derive(Debug, Default, Clone, Copy)]
struct FlushScore {
    aldh2_risk: u32,
    aldh2_observed: bool, // false → Flush tier should be Inconclusive
    adh1b_risk: u32,
    adh1b_observed: u32,
    adh1b_expected: u32,
}

#[derive(Debug, Default, Clone, Copy)]
struct AxisScore {
    risk: u32,
    observed: u32,
    expected: u32,
}

impl AxisScore {
    fn fully_covered(&self) -> bool {
        self.observed == self.expected
    }
}

fn score_sample(
    results: &[(&'static SnpTarget, Option<GenotypeCall>)],
) -> (FlushScore, AxisScore, AxisScore) {
    let mut flush = FlushScore::default();
    let mut reward = AxisScore::default();
    let mut histamine = AxisScore::default();

    for (target, call) in results {
        let confident = call.as_ref().filter(|c| c.confident);
        let copies = confident.map_or(0, |c| c.risk_copies);

        match target.axis {
            Axis::Flush if target.rsid == "rs671" => {
                if confident.is_some() {
                    flush.aldh2_observed = true;
                    flush.aldh2_risk = copies;
                }
            }
            Axis::Flush => {
                flush.adh1b_expected += 1;
                if confident.is_some() {
                    flush.adh1b_observed += 1;
                    flush.adh1b_risk += copies;
                }
            }
            Axis::Reward => {
                reward.expected += 1;
                if confident.is_some() {
                    reward.observed += 1;
                    reward.risk += copies;
                }
            }
            Axis::Histamine => {
                histamine.expected += 1;
                if confident.is_some() {
                    histamine.observed += 1;
                    histamine.risk += copies;
                }
            }
        }
    }

    (flush, reward, histamine)
}

fn print_axis_table(
    results: &[(&'static SnpTarget, Option<GenotypeCall>)],
    axis: Axis,
) {
    println!("| Variant | Gene | Genotype | Risk copies | Effect |");
    println!("| :--- | :--- | :--- | :---: | :--- |");
    for (target, call) in results.iter().filter(|(t, _)| t.axis == axis) {
        let (gt, copies) = match call {
            Some(c) if c.confident => (
                format!("`{}`", c.alleles.join("/")),
                c.risk_copies.to_string(),
            ),
            Some(c) => (
                format!("`{}` ⚠️", c.alleles.join("/")),
                "—".to_string(),
            ),
            None => ("—".to_string(), "—".to_string()),
        };
        println!(
            "| `{}` | *{}* | {} | {} | {} |",
            target.rsid, target.gene, gt, copies, target.short
        );
    }
    println!();
}

fn print_science_section() {
    println!("### How this works");
    println!();
    println!(
        "Alcohol metabolism is a two-step pipeline. **ADH** (alcohol dehydrogenase) \
         turns ethanol into **acetaldehyde** — a toxic intermediate responsible for \
         flushing, headaches, nausea, and rapid heart rate. **ALDH2** then converts \
         acetaldehyde into harmless acetate. The flush axis measures how much \
         acetaldehyde you accumulate (faster ADH1B in, slower ALDH2 out → more flush)."
    );
    println!();
    println!(
        "Whether you *like* drinking is a different question. The **mu-opioid \
         receptor** (OPRM1) and **dopamine system** (DRD2/ANKK1) determine how \
         rewarding alcohol feels. **GABA-A** receptors govern its anxiolytic \
         effect — the chill. The like axis stacks these three reward pathways."
    );
    println!();
    println!(
        "Drinks aren't pure ethanol. Wine, beer, and fermented anything carry \
         **histamine** (from bacterial decarboxylation during fermentation) — \
         and your body clears it via two enzymes: **DAO** (gut/blood, encoded \
         by AOC1) and **HNMT** (CNS/airways). Variants in either pathway slow \
         clearance, and histamine then drives the wine headache, the stuffy \
         nose, the racing heart, the post-drink itch. The junk axis stacks DAO \
         and HNMT loss-of-function variants. *Sulfites and tyramine don't have \
         clean common SNPs* — those phenotypes exist but the panel-level \
         genetics isn't there yet."
    );
    println!();
}

fn print_fine_print() {
    println!("### Fine print");
    println!();
    println!(
        "Eleven SNPs is a long way from the whole story. Body weight, gut \
         microbiome, sleep, food in your stomach, history of drinking, sulfite \
         and tyramine and tannin loads, hormones, and dozens of other genes \
         all shape your response. This is a toy demo, not medical advice. \
         Don't drink your way around your genotype."
    );
    println!();
}

fn print_further_reading() {
    println!("### Further reading");
    println!();
    println!(
        "- Brooks PJ *et al.* (2009). \"The Alcohol Flushing Response: An \
         Unrecognized Risk Factor for Esophageal Cancer.\" *PLoS Med* 6(3):e50."
    );
    println!(
        "- Edenberg HJ (2007). \"The Genetics of Alcohol Metabolism.\" \
         *Alcohol Research & Health* 30(1):5–13."
    );
    println!(
        "- Ray LA, Barr CS, Blendy JA, Oslin D, Goldman D, Anton RF (2012). \
         \"The role of the OPRM1 gene in alcohol use disorder and treatment \
         response.\" *Addiction Biology* 17(3):525–540."
    );
    println!(
        "- Edenberg HJ *et al.* (2004). \"Variations in GABRA2, encoding the α2 \
         subunit of the GABA-A receptor, are associated with alcohol \
         dependence and with brain oscillations.\" *Am J Hum Genet* 74(4):705–714."
    );
    println!(
        "- Maintz L, Yu CF, Rodríguez E, *et al.* (2011). \"Association of \
         single nucleotide polymorphisms in the diamine oxidase gene with \
         diamine oxidase serum activities.\" *Allergy* 66(7):893–902."
    );
    println!(
        "- Hrubisko M *et al.* (2021). \"Histamine intolerance — the more we know, \
         the less we know. A review.\" *Nutrients* 13(7):2228."
    );
    println!();
}

fn print_technical_details(results: &[(&'static SnpTarget, Option<GenotypeCall>)]) {
    println!("### Technical details");
    println!();
    println!(
        "Resolved through the `rsids/` lens: each rsID maps to a coordinate and your \
         genotype there, build-independently, without the app reading the variant file."
    );
    println!();
    println!("| rsID | Gene | Locus | Reference | Status |");
    println!("| :--- | :--- | :--- | :--- | :--- |");
    for (target, call) in results {
        let (locus, reference, status) = match call {
            Some(c) => (
                if c.chromosome.is_empty() || c.position.is_empty() {
                    "—".to_string()
                } else {
                    format!("`{}:{}`", c.chromosome, c.position)
                },
                if c.reference.is_empty() {
                    "—".to_string()
                } else {
                    format!("`{}`", c.reference)
                },
                if c.confident { "✓ found" } else { "⚠️ uncertain" },
            ),
            None => ("—".to_string(), "—".to_string(), "⚠️ not covered"),
        };
        println!(
            "| `{}` | *{}* | {} | {} | {} |",
            target.rsid, target.gene, locus, reference, status
        );
    }
    println!();
    println!("#### What each SNP does");
    println!();
    for target in TARGETS {
        println!(
            "- **`{}`** ({}, *{}*) — {}",
            target.rsid,
            match target.axis {
                Axis::Flush => "Flush",
                Axis::Reward => "Like",
                Axis::Histamine => "Junk",
            },
            target.gene,
            target.blurb
        );
    }
    println!();
}

// ── Entry point ────────────────────────────────────────────────────────────

const METADATA: &str = "\
[package]
name = \"drunk-o-type\"
version = \"0.2.0\"
datasets = [
    \"v1/genome/rsids/rs671\",
    \"v1/genome/rsids/rs1229984\",
    \"v1/genome/rsids/rs2066702\",
    \"v1/genome/rsids/rs1799971\",
    \"v1/genome/rsids/rs1800497\",
    \"v1/genome/rsids/rs279858\",
    \"v1/genome/rsids/rs10156191\",
    \"v1/genome/rsids/rs1049742\",
    \"v1/genome/rsids/rs1049793\",
    \"v1/genome/rsids/rs2052129\",
    \"v1/genome/rsids/rs11558538\",
]
";

// Resolve one rsID target through the `rsids/` lens: `base` is the per-rsID
// directory (`rsids/<rs#>`). The `genotype` leaf exists only where your calls
// cover the site; its absence means uncovered (or the catalog lacks the rsID).
fn resolve_target(base: &Path, target: &SnpTarget) -> Option<GenotypeCall> {
    let genotype = fs::read_to_string(base.join("genotype")).ok()?;
    let genotype = genotype.trim();

    // The lens returns alleles as bases (e.g. `A/C`, or `C|C` when phased), so
    // there is no REF/ALT index decoding to do; just count the risk base.
    let alleles: Vec<String> = genotype.split(['/', '|']).map(String::from).collect();
    let confident = !alleles.iter().any(|a| a == "." || a.is_empty());
    let risk_copies = if confident {
        alleles.iter().filter(|a| a.as_str() == target.risk_allele).count() as u32
    } else {
        0
    };

    let leaf = |name: &str| {
        fs::read_to_string(base.join(name))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    Some(GenotypeCall {
        alleles,
        risk_copies,
        confident,
        chromosome: leaf("chromosome"),
        position: leaf("position"),
        reference: leaf("reference"),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::args().len() < 2 {
        print!("{}", METADATA);
        return Ok(());
    }

    let dir = std::env::args().nth(1).unwrap();
    let rsids = Path::new(&dir).join("v1/genome/rsids");

    print_app_header();

    let results: Vec<(&'static SnpTarget, Option<GenotypeCall>)> = TARGETS
        .iter()
        .map(|t| (t, resolve_target(&rsids.join(t.rsid), t)))
        .collect();

    print_sample_report(&results);
    print_science_section();
    print_fine_print();
    print_further_reading();
    print_technical_details(&results);

    Ok(())
}
