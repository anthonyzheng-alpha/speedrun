// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Seeds freshly created collections with starter MCAT study decks.
//!
//! This is invoked from [`crate::collection::CollectionBuilder::build`] only
//! when the collection was just created and seeding has been explicitly
//! enabled (see `set_seed_initial_content`). Tests that build collections
//! directly leave seeding disabled, so their collections remain empty.

use anki_proto::notetypes::stock_notetype::OriginalStockKind;

use crate::prelude::*;

/// A single front/back flashcard.
type Card = (&'static str, &'static str);

/// A deck name (in native `::`-separated form) paired with its starter cards.
type Section = (&'static str, &'static [Card]);

/// Curated MCAT starter content, grouped by the four scored sections of the
/// exam. Users are free to edit, delete, or expand these after import.
const MCAT_SECTIONS: &[Section] = &[
    (
        "MCAT::Biology & Biochemistry",
        &[
            (
                "What are the four levels of protein structure?",
                "Primary (amino acid sequence), secondary (alpha helices / beta sheets), tertiary (3D folding), and quaternary (multiple subunits).",
            ),
            (
                "Which enzyme unwinds the DNA double helix during replication?",
                "Helicase.",
            ),
            (
                "What is the role of the enzyme telomerase?",
                "It adds repetitive nucleotide sequences to the ends of chromosomes (telomeres) to prevent their shortening during replication.",
            ),
            (
                "During which phase of the cell cycle is DNA replicated?",
                "S phase (synthesis phase) of interphase.",
            ),
            (
                "What is the net ATP yield of glycolysis per glucose molecule?",
                "2 ATP (net), along with 2 NADH and 2 pyruvate.",
            ),
            (
                "Where does the citric acid (Krebs) cycle occur in eukaryotes?",
                "The mitochondrial matrix.",
            ),
            (
                "What type of bond links amino acids together in a protein?",
                "A peptide (amide) bond.",
            ),
            (
                "What is the difference between competitive and noncompetitive enzyme inhibition?",
                "Competitive inhibitors bind the active site (raising Km, Vmax unchanged); noncompetitive inhibitors bind an allosteric site (Vmax decreases, Km unchanged).",
            ),
            (
                "Which hormone lowers blood glucose, and where is it produced?",
                "Insulin, produced by the beta cells of the pancreatic islets of Langerhans.",
            ),
            (
                "What is the central dogma of molecular biology?",
                "DNA is transcribed into RNA, which is translated into protein (DNA -> RNA -> protein).",
            ),
        ],
    ),
    (
        "MCAT::Chemistry & Physics",
        &[
            (
                "State Newton's second law of motion.",
                "F = ma; the net force on an object equals its mass times its acceleration.",
            ),
            (
                "What is the ideal gas law?",
                "PV = nRT, where P is pressure, V is volume, n is moles, R is the gas constant, and T is absolute temperature.",
            ),
            (
                "Define pH in terms of hydrogen ion concentration.",
                "pH = -log10[H+].",
            ),
            (
                "What distinguishes an exothermic from an endothermic reaction?",
                "Exothermic reactions release heat (negative delta H); endothermic reactions absorb heat (positive delta H).",
            ),
            (
                "What is the difference between a strong and weak acid?",
                "A strong acid fully dissociates in water; a weak acid only partially dissociates (has an equilibrium described by Ka).",
            ),
            (
                "State Ohm's law.",
                "V = IR; voltage equals current times resistance.",
            ),
            (
                "What is the SI unit of force and how is it defined?",
                "The newton (N), equal to 1 kg*m/s^2.",
            ),
            (
                "What does a catalyst do to a chemical reaction?",
                "It lowers the activation energy, increasing the reaction rate without being consumed; it does not change the equilibrium position.",
            ),
            (
                "What is the relationship between wavelength and frequency of light?",
                "c = (lambda)(f); the speed of light equals wavelength times frequency, so they are inversely related.",
            ),
            (
                "Define an enantiomer.",
                "Stereoisomers that are nonsuperimposable mirror images of each other.",
            ),
        ],
    ),
    (
        "MCAT::Psychology & Sociology",
        &[
            (
                "What is classical conditioning?",
                "Learning in which a neutral stimulus becomes associated with a meaningful stimulus to elicit a conditioned response (e.g., Pavlov's dogs).",
            ),
            (
                "Name Maslow's hierarchy of needs from bottom to top.",
                "Physiological, safety, love/belonging, esteem, and self-actualization.",
            ),
            (
                "What is the difference between an independent and dependent variable?",
                "The independent variable is manipulated by the researcher; the dependent variable is measured as the outcome.",
            ),
            (
                "Define the fundamental attribution error.",
                "The tendency to attribute others' behavior to internal dispositions while underestimating situational factors.",
            ),
            (
                "What is operant conditioning?",
                "Learning in which behavior is shaped by its consequences (reinforcement increases behavior; punishment decreases it).",
            ),
            (
                "Distinguish between assimilation and accommodation (Piaget).",
                "Assimilation incorporates new information into existing schemas; accommodation modifies schemas to fit new information.",
            ),
            (
                "What is socialization?",
                "The lifelong process by which individuals learn and internalize the norms, values, and behaviors of their society.",
            ),
            (
                "Define social facilitation.",
                "The tendency for the presence of others to improve performance on simple/well-learned tasks and impair it on complex/novel tasks.",
            ),
            (
                "What are the stages of sleep characterized by EEG patterns?",
                "Stage 1 (theta), Stage 2 (sleep spindles, K-complexes), Stages 3-4 (delta/slow-wave), and REM sleep.",
            ),
            (
                "What is the difference between a primary and secondary group?",
                "Primary groups are small, intimate, and long-lasting (e.g., family); secondary groups are larger, impersonal, and goal-oriented (e.g., coworkers).",
            ),
        ],
    ),
    (
        "MCAT::Critical Analysis & Reasoning (CARS)",
        &[
            (
                "What does the CARS section test?",
                "Reading comprehension, analysis, and reasoning skills using passages from the humanities and social sciences; no outside content knowledge is required.",
            ),
            (
                "What is the difference between an inference and a stated detail?",
                "A stated detail is explicitly written in the passage; an inference is a logical conclusion drawn from what is stated but not directly written.",
            ),
            (
                "What is the main idea (thesis) of a passage?",
                "The central point or argument the author is trying to convey, around which the supporting details are organized.",
            ),
            (
                "What is the author's tone?",
                "The author's attitude toward the subject, conveyed through word choice (e.g., critical, neutral, enthusiastic, skeptical).",
            ),
            (
                "What strategy helps with 'strengthen' or 'weaken' questions?",
                "Identify the author's core argument, then evaluate which answer choice provides evidence that supports or undermines that specific claim.",
            ),
            (
                "How should you approach answer choices with extreme wording (e.g., 'always', 'never')?",
                "Treat them with caution; absolute statements are often too strong to be supported by a nuanced passage.",
            ),
            (
                "What is the purpose of an 'except' or 'least' question?",
                "It asks you to identify the one answer choice that does NOT fit; the other three will be true or supported by the passage.",
            ),
            (
                "Why is it important to read for structure, not just content, in CARS?",
                "Understanding how an argument is built (claims, evidence, counterarguments) helps answer reasoning questions about the author's purpose and logic.",
            ),
        ],
    ),
];

impl Collection {
    /// Seed a freshly created collection with starter MCAT decks and cards.
    pub(crate) fn seed_mcat_decks(&mut self) -> Result<()> {
        self.transact(Op::AddNote, |col| {
            let notetype = col.basic_notetype_for_seeding()?;
            for (deck_name, cards) in MCAT_SECTIONS {
                let mut deck = Deck::new_normal();
                deck.name = NativeDeckName::from_native_str(*deck_name);
                col.add_deck_inner(&mut deck, col.usn()?)?;
                let deck_id = deck.id;
                for (front, back) in *cards {
                    let mut note = notetype.new_note();
                    let fields = note.fields_mut();
                    if fields.len() >= 2 {
                        fields[0] = (*front).to_string();
                        fields[1] = (*back).to_string();
                    }
                    col.add_note_inner(&mut note, deck_id)?;
                }
            }
            Ok(())
        })
        .map(|_| ())
    }

    /// Resolve the stock "Basic" notetype regardless of the current UI
    /// language, falling back to the first available notetype if the stock
    /// kind cannot be found.
    fn basic_notetype_for_seeding(&mut self) -> Result<std::sync::Arc<Notetype>> {
        let notetypes = self.get_all_notetypes()?;
        let basic = notetypes
            .iter()
            .find(|nt| nt.config.original_stock_kind() == OriginalStockKind::Basic)
            .or_else(|| notetypes.first())
            .cloned()
            .or_invalid("no notetypes available to seed MCAT decks")?;
        Ok(basic)
    }
}
