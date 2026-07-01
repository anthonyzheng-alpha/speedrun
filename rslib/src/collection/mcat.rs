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
            (
                "What four groups are attached to the central alpha carbon of an amino acid?",
                "An amino group, a carboxylic acid group, a hydrogen atom, and a variable R group. The R group determines the amino acid's properties.",
            ),
            (
                "What is the isoelectric point (pI) of an amino acid?",
                "The pH at which the amino acid exists as a neutral zwitterion with no net charge. For a neutral side chain, pI is the average of the two relevant pKa values.",
            ),
            (
                "Which amino acid is the only achiral one, and what configuration do the chiral ones have?",
                "Glycine is the only achiral amino acid. All chiral amino acids in eukaryotic proteins are L (S configuration) except cysteine, which is L but R.",
            ),
            (
                "What stabilizes the secondary structure of a protein?",
                "Hydrogen bonding between backbone amino (N-H) and nonadjacent carbonyl (C=O) groups. Common forms are the alpha helix and beta pleated sheet.",
            ),
            (
                "What bond forms cystine, and how does it stabilize tertiary structure?",
                "A disulfide bond forms when two cysteine thiol groups are oxidized, creating a covalent link. This helps stabilize a protein's three dimensional shape.",
            ),
            (
                "Why does burying hydrophobic R groups in a protein's core favor folding?",
                "It releases ordered water from the solvation layer, increasing the entropy of surrounding water. This makes the overall Gibbs free energy of folding negative.",
            ),
            (
                "Do enzymes change the delta G or equilibrium of a reaction?",
                "No. Enzymes only lower activation energy and speed up the rate; they do not alter delta G, delta H, or the final equilibrium position. They catalyze both forward and reverse reactions.",
            ),
            (
                "What does the Michaelis constant Km represent?",
                "Km is the substrate concentration at which an enzyme operates at half its Vmax. A lower Km means higher affinity for the substrate.",
            ),
            (
                "How does competitive inhibition affect Vmax and Km?",
                "Vmax is unchanged and Km increases. The inhibitor resembles the substrate and binds the active site, so it can be overcome by adding more substrate.",
            ),
            (
                "How does noncompetitive inhibition affect Vmax and Km?",
                "Vmax decreases and Km is unchanged. The inhibitor binds equally well to the free enzyme and the enzyme-substrate complex at an allosteric site.",
            ),
            (
                "How does uncompetitive inhibition affect Vmax and Km?",
                "Both Vmax and Km decrease. The inhibitor binds only to the enzyme-substrate complex.",
            ),
            (
                "What is a zymogen?",
                "An inactive enzyme precursor that is secreted in an inactive form and later activated by cleavage. Examples include pepsinogen and trypsinogen.",
            ),
            (
                "What does phosphorylation do to catabolic versus anabolic enzymes?",
                "In catabolism the phosphorylated form is usually active; in anabolism the phosphorylated form is usually inactive. This lets hormones coordinate opposing pathways.",
            ),
            (
                "State Chargaff's rules for DNA base composition.",
                "The amount of purines equals the amount of pyrimidines, so A equals T and G equals C. Purines are adenine and guanine; pyrimidines are cytosine, thymine, and uracil.",
            ),
            (
                "What does it mean that DNA replication is semiconservative?",
                "Each new double helix contains one original parental strand and one newly synthesized strand. Replication occurs during the S phase of the cell cycle.",
            ),
            (
                "How does nucleotide excision repair fix thymine dimers?",
                "An excision endonuclease removes a short stretch of the helix-distorting lesion in a cut-and-patch process. DNA polymerase and ligase then fill and seal the gap.",
            ),
            (
                "What are telomeres and how are they maintained?",
                "Telomeres are repetitive GC-rich ends of chromosomes that prevent DNA unraveling and are shortened each replication. Telomerase can partially restore them.",
            ),
            (
                "What are the start and stop codons of the genetic code?",
                "The start codon is AUG, which codes for methionine. The three stop codons are UAA, UGA, and UAG.",
            ),
            (
                "What does it mean that the genetic code is degenerate?",
                "Multiple codons can encode the same amino acid, often differing only in the third wobble base. This lets many point mutations be silent.",
            ),
            (
                "What happens to introns and exons during mRNA processing?",
                "Introns are spliced out and remain in the nucleus, while exons are joined and exported as mature mRNA. A 5-prime cap and poly-A tail are also added for stability.",
            ),
            (
                "In translation, what happens at the A, P, and E sites of the ribosome?",
                "A new aminoacyl-tRNA enters the A site, the growing polypeptide is held in the P site, and the uncharged tRNA exits from the E site. The A site tRNA then shifts to the P site.",
            ),
            (
                "How does an inducible operon like the lac operon work?",
                "Normally a repressor is bound to the operator, blocking transcription. An inducer binds and removes the repressor, turning the genes on.",
            ),
            (
                "How does a repressible operon like the trp operon work?",
                "The genes are normally transcribed, but a corepressor binds the repressor and the complex binds the operator to shut transcription off. High tryptophan acts as the corepressor.",
            ),
            (
                "Contrast oncogenes and tumor suppressor genes.",
                "Oncogenes arise from mutated proto-oncogenes and promote cell cycling, like a stuck gas pedal. Mutated tumor suppressor genes fail to slow division or repair DNA, like cut brakes.",
            ),
            (
                "State Mendel's law of segregation.",
                "An organism has two alleles per gene that separate during anaphase I of meiosis. As a result each gamete carries only one allele for a trait.",
            ),
            (
                "What is the difference between codominance and incomplete dominance?",
                "In codominance both alleles are fully and separately expressed, like AB blood type. In incomplete dominance the heterozygote shows a blended intermediate phenotype.",
            ),
            (
                "Give the Hardy-Weinberg equations and what p and q represent.",
                "p + q = 1 and p^2 + 2pq + q^2 = 1, where p is the dominant allele frequency and q is the recessive allele frequency. They hold only when a population is not evolving.",
            ),
            (
                "What experiment confirmed that DNA, not protein, is the genetic material?",
                "The Hershey-Chase experiment showed that only radiolabeled DNA from bacteriophages entered infected bacteria. Griffith and Avery-MacLeod-McCarty earlier pointed to DNA via transformation.",
            ),
            (
                "What is the net yield of glycolysis per glucose?",
                "Glucose plus 2 NAD+ and 2 ADP yields 2 pyruvate, 2 ATP, and 2 NADH. It occurs in the cytoplasm.",
            ),
            (
                "What is the rate-limiting enzyme of glycolysis and how is it regulated?",
                "Phosphofructokinase-1 (PFK-1) is rate limiting. It is activated by AMP and fructose 2,6-bisphosphate and inhibited by ATP and citrate.",
            ),
            (
                "Where does the citric acid cycle occur and what does it produce per acetyl-CoA?",
                "It occurs in the mitochondrial matrix. Each acetyl-CoA yields 3 NADH, 1 FADH2, 1 GTP, and 2 CO2.",
            ),
            (
                "What is the rate-limiting enzyme of the citric acid cycle?",
                "Isocitrate dehydrogenase. It is inhibited by ATP and NADH and activated by ADP and NAD+.",
            ),
            (
                "How does ATP synthase generate ATP in oxidative phosphorylation?",
                "The proton-motive force drives H+ back into the matrix through the Fo channel, and the F1 portion uses that energy to phosphorylate ADP into ATP. This is chemiosmotic coupling.",
            ),
            (
                "What is the final electron acceptor of the electron transport chain?",
                "Oxygen, which has the highest reduction potential, accepts electrons at Complex IV to form water. Complexes I, III, and IV pump protons across the inner membrane.",
            ),
            (
                "Which enzymes bypass the irreversible steps of glycolysis in gluconeogenesis?",
                "Pyruvate carboxylase and PEPCK bypass pyruvate kinase, fructose-1,6-bisphosphatase bypasses PFK-1, and glucose-6-phosphatase bypasses hexokinase. It occurs mainly in the liver.",
            ),
            (
                "What glycosidic bonds do glycogen synthase and the branching enzyme create?",
                "Glycogen synthase forms alpha-1,4 bonds along the chain, and the branching enzyme forms alpha-1,6 bonds at branch points. Glycogen is stored mainly in liver and muscle.",
            ),
            (
                "What is the main product and rate-limiting enzyme of the pentose phosphate pathway?",
                "It produces NADPH and sugars for biosynthesis in the cytoplasm. Glucose-6-phosphate dehydrogenase (G6PD) is the rate-limiting enzyme.",
            ),
            (
                "Describe the steps of beta-oxidation of fatty acids.",
                "In the mitochondria, cycles of oxidation, hydration, oxidation, and thiolytic cleavage shorten the fatty acid by two carbons each round. Each cycle yields FADH2, NADH, and acetyl-CoA.",
            ),
            (
                "When and where are ketone bodies produced?",
                "They form in the liver via ketogenesis when excess acetyl-CoA accumulates during prolonged starvation. The brain can derive up to two-thirds of its energy from ketone bodies during starvation.",
            ),
            (
                "Which amino acids are solely ketogenic?",
                "Leucine and lysine are the only purely ketogenic amino acids. All other amino acids are at least partly glucogenic.",
            ),
            (
                "Why is ATP hydrolysis energetically favorable?",
                "Its high-energy phosphate bonds are stabilized upon hydrolysis by resonance, ionization, and relief of charge repulsion, giving a large negative delta G. ATP is considered a mid-level energy carrier.",
            ),
            (
                "What organelles define eukaryotic cells and where does the ETC occur in each cell type?",
                "Eukaryotes have membrane-bound organelles including a nucleus and mitochondria, and run the ETC on the inner mitochondrial membrane. Prokaryotes lack these organelles and run the ETC in the cell membrane.",
            ),
            (
                "Compare the rough and smooth endoplasmic reticulum.",
                "The rough ER is studded with ribosomes and makes secreted and membrane proteins. The smooth ER makes lipids and handles detoxification.",
            ),
            (
                "Distinguish gap junctions, tight junctions, and desmosomes.",
                "Gap junctions allow rapid exchange of ions between cells, tight junctions seal the space between cells to block leakage, and desmosomes anchor adjacent cells via their cytoskeletons.",
            ),
            (
                "How does the Na+/K+ pump move ions and what type of transport is it?",
                "It pumps 3 Na+ out and 2 K+ in against their gradients using ATP, making it primary active transport. This maintains the resting membrane potential.",
            ),
            (
                "Distinguish gram-positive and gram-negative bacteria.",
                "Gram-positive bacteria stain purple and have a thick peptidoglycan cell wall. Gram-negative bacteria stain pink-red with a thin peptidoglycan layer plus an outer membrane.",
            ),
            (
                "What is a retrovirus and what enzyme does it require?",
                "A retrovirus is a single-stranded RNA virus that uses reverse transcriptase to make DNA from its RNA genome. That DNA can then integrate into the host genome.",
            ),
            (
                "Contrast the lytic and lysogenic bacteriophage life cycles.",
                "In the lytic cycle virions are produced until the host cell bursts. In the lysogenic cycle the viral genome integrates as a prophage and stays dormant until stress activates it.",
            ),
            (
                "What are the three primary germ layers and one derivative of each?",
                "Ectoderm forms skin and nervous system, mesoderm forms muscle, bone, and the circulatory system, and endoderm forms the gut lining and internal organs. They arise during gastrulation.",
            ),
            (
                "Which neurotransmitters do the two divisions of the autonomic nervous system use?",
                "All preganglionic neurons of both divisions release acetylcholine. Sympathetic (fight-or-flight) postganglionic neurons then release norepinephrine at target organs, while parasympathetic (rest-and-digest) postganglionic neurons release acetylcholine. Epinephrine is released as a hormone from the adrenal medulla, not at a postganglionic synapse.",
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
                "What distinguishes an exothermic from an endothermic reaction?",
                "Exothermic reactions release heat (negative delta H); endothermic reactions absorb heat (positive delta H).",
            ),
            (
                "What is the difference between a strong and weak acid?",
                "A strong acid fully dissociates in water; a weak acid only partially dissociates (has an equilibrium described by Ka).",
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
                "How is pH defined in terms of hydrogen ion concentration?",
                "pH = -log[H+]. A one-unit drop in pH corresponds to a tenfold increase in [H+], and at 25 C a neutral solution has pH 7.",
            ),
            (
                "What is the relationship between pH and pOH at 25 C?",
                "pH + pOH = 14, because Kw = [H+][OH-] = 1.0 x 10^-14 at 25 C.",
            ),
            (
                "What does the Henderson-Hasselbalch equation state for a buffer?",
                "pH = pKa + log([A-]/[HA]). When [A-] equals [HA], the pH equals the pKa.",
            ),
            (
                "What makes a solution a good buffer and where is it most effective?",
                "A buffer contains comparable amounts of a weak acid and its conjugate base, and it resists pH change most effectively within about one pH unit of the pKa.",
            ),
            (
                "Name the common strong acids that fully dissociate in water.",
                "HCl, HBr, HI, HNO3, H2SO4, and HClO4 are strong acids that dissociate essentially completely.",
            ),
            (
                "What relationship links Ka and Kb of a conjugate acid-base pair?",
                "Ka x Kb = Kw = 1.0 x 10^-14 at 25 C, so a stronger acid has a weaker conjugate base.",
            ),
            (
                "What are the conditions defined as STP and the molar volume of an ideal gas there?",
                "STP is 0 C (273.15 K) and 1 atm; one mole of ideal gas occupies about 22.4 L under these conditions.",
            ),
            (
                "What does Dalton's law of partial pressures state?",
                "The total pressure of a gas mixture equals the sum of the partial pressures of its components, and each partial pressure equals its mole fraction times total pressure.",
            ),
            (
                "How do electronegativity and atomic radius trend across the periodic table?",
                "Electronegativity increases up and to the right, while atomic radius increases down and to the left toward larger, less tightly held atoms.",
            ),
            (
                "How does first ionization energy trend on the periodic table?",
                "Ionization energy generally increases across a period and decreases down a group, mirroring electronegativity and opposing atomic size.",
            ),
            (
                "What do the four quantum numbers n, l, ml, and ms describe?",
                "n is the principal energy level, l is the subshell shape, ml is the orbital orientation, and ms is the electron spin (+1/2 or -1/2).",
            ),
            (
                "What does Le Chatelier's principle predict when a system at equilibrium is stressed?",
                "The system shifts to counteract the change; adding reactant or removing product shifts it toward products, and vice versa.",
            ),
            (
                "How is the equilibrium constant Keq written for aA + bB -> cC + dD?",
                "Keq = ([C]^c [D]^d) / ([A]^a [B]^b), using equilibrium concentrations; pure solids and liquids are omitted.",
            ),
            (
                "What is the reaction quotient Q used for?",
                "Q has the same form as Keq but uses current concentrations; if Q < Keq the reaction proceeds forward, and if Q > Keq it proceeds in reverse.",
            ),
            (
                "What does the order of a reaction tell you about rate dependence?",
                "The order in a reactant is the exponent on its concentration in the rate law; overall order is the sum of exponents and is determined experimentally, not from stoichiometry.",
            ),
            (
                "What determines whether a reaction is spontaneous according to Gibbs free energy?",
                "delta G = delta H - T(delta S); a reaction is spontaneous when delta G is negative.",
            ),
            (
                "When is a reaction spontaneous at all temperatures based on enthalpy and entropy?",
                "When delta H is negative and delta S is positive, delta G is negative at every temperature, so the reaction is always spontaneous.",
            ),
            (
                "What does Hess's law allow you to do?",
                "It states that the total enthalpy change of a reaction is the sum of the enthalpy changes of its steps, since enthalpy is a state function independent of path.",
            ),
            (
                "In a galvanic (voltaic) cell, what happens at the anode and cathode?",
                "Oxidation occurs at the anode and reduction occurs at the cathode; electrons flow from anode to cathode through the external wire.",
            ),
            (
                "How is cell potential related to spontaneity and free energy?",
                "delta G = -nFE; a positive cell EMF corresponds to negative delta G and a spontaneous (galvanic) reaction.",
            ),
            (
                "What is the sign convention for the electrodes in an electrolytic cell?",
                "In electrolysis the anode is positive and the cathode is negative, and a non-spontaneous reaction is driven by an external power source.",
            ),
            (
                "State the basic rules for assigning oxidation numbers.",
                "Free elements are 0, oxygen is usually -2, hydrogen is usually +1, and the sum of oxidation numbers equals the overall charge of the species.",
            ),
            (
                "What are colligative properties and give examples?",
                "Colligative properties depend on the number of solute particles, not their identity; examples include boiling point elevation, freezing point depression, and osmotic pressure.",
            ),
            (
                "What does the solubility product Ksp represent?",
                "Ksp is the equilibrium constant for a slightly soluble salt dissolving into its ions; a solid precipitates when the ion product exceeds Ksp.",
            ),
            (
                "What is the common ion effect?",
                "Adding an ion already present in a solubility equilibrium shifts it toward the solid, decreasing the salt's solubility (a Le Chatelier effect).",
            ),
            (
                "What defines the equivalence point of an acid-base titration?",
                "It is where moles of added titrant exactly neutralize the analyte; for a strong acid-strong base titration the equivalence point pH is 7.",
            ),
            (
                "How do you identify a carbonyl group and name two functional groups containing it?",
                "A carbonyl is a C double bonded to O; aldehydes have the carbonyl at a chain end (bonded to H) while ketones have it between two carbons.",
            ),
            (
                "What are enantiomers and how do they differ physically?",
                "Enantiomers are non-superimposable mirror-image stereoisomers; they share most physical properties but rotate plane-polarized light in opposite directions.",
            ),
            (
                "What characterizes an SN2 reaction mechanism?",
                "SN2 is a one-step, concerted reaction that is second order overall, favored by strong nucleophiles and unhindered (methyl or primary) substrates, and it inverts stereochemistry.",
            ),
            (
                "What characterizes an SN1 reaction mechanism?",
                "SN1 is a two-step reaction, first order in substrate, that proceeds through a carbocation intermediate; it is favored by tertiary substrates and gives racemization.",
            ),
            (
                "What is the difference between a nucleophile and an electrophile?",
                "A nucleophile is electron-rich and donates an electron pair, while an electrophile is electron-poor and accepts an electron pair.",
            ),
            (
                "How does resonance affect molecular stability?",
                "Resonance delocalizes electrons over multiple structures, and greater delocalization lowers energy and increases stability (as in carboxylate ions and benzene).",
            ),
            (
                "What is the difference between constitutional isomers and stereoisomers?",
                "Constitutional (structural) isomers differ in atom connectivity, whereas stereoisomers have the same connectivity but differ in spatial arrangement.",
            ),
            (
                "What products form when primary versus secondary alcohols are oxidized?",
                "Primary alcohols oxidize to aldehydes and then carboxylic acids, while secondary alcohols oxidize to ketones; tertiary alcohols resist oxidation.",
            ),
            (
                "Which IR absorptions are diagnostic for O-H and C=O groups?",
                "A broad O-H stretch appears around 3200 to 3550 per cm, and a strong C=O carbonyl stretch appears near 1700 per cm.",
            ),
            (
                "Give the kinematic equation relating displacement, initial velocity, acceleration, and time.",
                "x = v0*t + (1/2)a*t^2 for constant acceleration; another useful form is v^2 = v0^2 + 2a*x.",
            ),
            (
                "How is work defined for a constant force?",
                "Work = F*d*cos(theta), where theta is the angle between the force and displacement; force perpendicular to motion does zero work.",
            ),
            (
                "What is the work-energy theorem?",
                "The net work done on an object equals its change in kinetic energy, W_net = delta KE, where KE = (1/2)m*v^2.",
            ),
            (
                "What does conservation of mechanical energy state?",
                "In the absence of non-conservative forces like friction, the sum of kinetic and potential energy stays constant, so KE + PE is conserved.",
            ),
            (
                "How are impulse and momentum related?",
                "Impulse equals the change in momentum, F*t = delta(m*v); momentum p = m*v is conserved in a system with no external net force.",
            ),
            (
                "State Coulomb's law for the force between two point charges.",
                "F = k*q1*q2 / r^2; the force is attractive for opposite charges and repulsive for like charges, and it falls off with the square of the distance.",
            ),
            (
                "How is electric field related to force on a charge?",
                "E = F/q, so the force on a charge is F = qE; the field points away from positive charges and toward negative charges.",
            ),
            (
                "State Ohm's law and the formula for electrical power.",
                "V = I*R relates voltage, current, and resistance; power dissipated is P = I*V = I^2*R = V^2/R.",
            ),
            (
                "How do resistors combine in series versus parallel?",
                "In series resistances add (R_total = R1 + R2 + ...); in parallel the reciprocals add (1/R_total = 1/R1 + 1/R2 + ...), giving a smaller total.",
            ),
            (
                "How is the capacitance of a capacitor defined?",
                "C = Q/V, the charge stored per unit voltage; energy stored is U = (1/2)C*V^2.",
            ),
            (
                "What is the relationship among wave speed, frequency, and wavelength?",
                "v = f * lambda; for a given medium the speed is fixed, so higher frequency means shorter wavelength.",
            ),
            (
                "What is the Doppler effect for sound?",
                "Observed frequency rises as source and observer approach and falls as they recede, because relative motion compresses or stretches the wavelengths.",
            ),
            (
                "How does sound intensity level in decibels relate to intensity?",
                "Level (dB) = 10*log(I/I0), so every 10 dB increase is a tenfold increase in intensity; I0 is the reference 1 x 10^-12 W/m^2.",
            ),
            (
                "State the thin lens and mirror equation and the sign of focal length.",
                "1/f = 1/do + 1/di; converging lenses and concave mirrors have positive focal length, while diverging lenses and convex mirrors have negative focal length.",
            ),
            (
                "State Snell's law of refraction.",
                "n1*sin(theta1) = n2*sin(theta2); light bends toward the normal when entering a higher-index (slower) medium.",
            ),
            (
                "What is total internal reflection and when does it occur?",
                "It occurs when light travels from a higher- to lower-index medium at an angle beyond the critical angle, causing all light to reflect back into the denser medium.",
            ),
            (
                "How is pressure in a static fluid related to depth?",
                "P = P0 + rho*g*h; pressure increases linearly with depth and depends on fluid density but not container shape.",
            ),
            (
                "State Archimedes' principle of buoyancy.",
                "The buoyant force on a submerged or floating object equals the weight of the fluid it displaces, F_b = rho_fluid * V_displaced * g.",
            ),
            (
                "What does the continuity equation state for an incompressible fluid?",
                "A1*v1 = A2*v2, so fluid speeds up when the cross-sectional area of the pipe narrows and slows when it widens.",
            ),
            (
                "What does Bernoulli's principle relate in a flowing fluid?",
                "Along a streamline, P + (1/2)rho*v^2 + rho*g*h is constant, so regions of faster flow have lower pressure.",
            ),
            (
                "What determines the period of a simple pendulum and a mass on a spring?",
                "A pendulum's period is T = 2*pi*sqrt(L/g) (independent of mass), and a spring's is T = 2*pi*sqrt(m/k); both describe simple harmonic motion.",
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
            (
                "What is the difference between sensation and perception?",
                "Sensation is the peripheral process of detecting stimuli and converting physical energy into neural signals (transduction). Perception is the central process of organizing and interpreting that sensory information to give it meaning.",
            ),
            (
                "What is an absolute threshold?",
                "The minimum stimulus intensity needed to detect a stimulus 50 percent of the time, such as the faintest sound you can hear or the dimmest light you can detect.",
            ),
            (
                "What is the difference threshold (JND) and what law governs it?",
                "The just-noticeable difference is the minimum change in stimulus intensity detectable 50 percent of the time. Weber's Law states the JND is a constant proportion of the original stimulus intensity.",
            ),
            (
                "How does top-down processing differ from bottom-up processing?",
                "Top-down processing is knowledge-driven, using prior expectations and context to interpret sensations. Bottom-up processing is data-driven, building perception from the raw features of the stimulus.",
            ),
            (
                "What is the difference between positive and negative reinforcement?",
                "Both increase behavior. Positive reinforcement adds a desirable stimulus after a behavior; negative reinforcement removes an aversive stimulus. Positive means adding, negative means removing.",
            ),
            (
                "What is the difference between positive and negative punishment?",
                "Both decrease behavior. Positive punishment adds an aversive stimulus; negative punishment removes a desirable stimulus. The terms refer to addition versus subtraction, not good versus bad.",
            ),
            (
                "How do stimulus generalization and discrimination differ in conditioning?",
                "Generalization is responding to stimuli similar to the conditioned stimulus, widening the range of effective stimuli. Discrimination is responding only to the specific conditioned stimulus, narrowing the response.",
            ),
            (
                "What is observational learning and who is it associated with?",
                "Learning by watching and imitating others rather than through direct reinforcement. It is central to Bandura's social learning theory, demonstrated by the Bobo doll experiment.",
            ),
            (
                "What is the difference between explicit and implicit memory?",
                "Explicit (declarative) memory is conscious memory for facts and events that can be verbally reported. Implicit (non-declarative) memory is unconscious, automatic memory for skills and procedures.",
            ),
            (
                "How do semantic and episodic memory differ?",
                "Both are explicit. Semantic memory stores general facts without personal context, while episodic memory stores personally experienced events tied to a specific time, place, and emotion.",
            ),
            (
                "How does short-term (working) memory differ from long-term memory?",
                "Short-term memory holds about 7 plus or minus 2 items for 15 to 30 seconds and is active and temporary. Long-term memory has theoretically unlimited capacity and durable storage.",
            ),
            (
                "What is the difference between proactive and retroactive interference?",
                "Proactive interference is when old memories disrupt new learning (old pushes forward). Retroactive interference is when new memories disrupt old ones (new reaches backward).",
            ),
            (
                "What is the difference between encoding and retrieval?",
                "Encoding converts information into a storable form (input), while retrieval accesses stored information and brings it into awareness (output). The tip-of-the-tongue phenomenon is a retrieval failure, not an encoding failure.",
            ),
            (
                "What distinguishes Piaget's concrete operational stage from the formal operational stage?",
                "In the concrete operational stage (about 7 to 11 years) children reason logically about tangible objects and grasp conservation. In the formal operational stage (12 plus years) they reason abstractly and hypothetically.",
            ),
            (
                "What does the Yerkes-Dodson (arousal) law state about motivation?",
                "Performance improves with arousal up to an optimal point, after which further arousal impairs performance, producing an inverted U-shaped relationship between arousal and performance.",
            ),
            (
                "What is the difference between intrinsic and extrinsic motivation?",
                "Intrinsic motivation arises from internal satisfaction or interest in the activity itself, while extrinsic motivation comes from external rewards or the avoidance of punishment.",
            ),
            (
                "How does the Schachter-Singer two-factor theory of emotion work?",
                "Emotion results from both physiological arousal and a cognitive label (interpretation) of that arousal. The same arousal can produce different emotions depending on how it is appraised.",
            ),
            (
                "How do the James-Lange and Cannon-Bard theories of emotion differ?",
                "James-Lange says emotion follows from physiological arousal (we feel afraid because we tremble). Cannon-Bard says physiological arousal and the emotional experience occur simultaneously and independently.",
            ),
            (
                "What are the three stages of the general adaptation syndrome?",
                "Selye's model of the stress response has three stages: alarm (initial fight-or-flight activation), resistance (sustained coping), and exhaustion (depleted resources and vulnerability to illness).",
            ),
            (
                "What is the difference between problem-focused and emotion-focused coping?",
                "Problem-focused coping targets the stressor itself by taking action to change or remove it. Emotion-focused coping manages the emotional distress caused by the stressor rather than the stressor itself.",
            ),
            (
                "What is the looking-glass self?",
                "Cooley's concept that we develop our self-concept based on how we imagine others perceive and judge us. Our sense of self is shaped by the reflected appraisals of others.",
            ),
            (
                "What is impression management (dramaturgy)?",
                "Goffman's idea that people consciously control the impressions others form of them, performing a front-stage self for audiences while a more authentic self exists back stage.",
            ),
            (
                "What is the difference between major depressive disorder and bipolar disorder?",
                "Both involve depressive episodes, but bipolar disorder additionally includes manic or hypomanic episodes. The presence of mania or hypomania distinguishes bipolar from unipolar MDD.",
            ),
            (
                "What is the diathesis-stress model of psychological disorders?",
                "The model proposes that disorders arise from the interaction of a predisposing vulnerability (diathesis) and environmental stressors, rather than from either factor alone.",
            ),
            (
                "What is the difference between conformity and compliance?",
                "Conformity is changing behavior to match a group without any direct request, driven by group norms. Compliance is changing behavior in response to a direct request from a peer with no authority over you.",
            ),
            (
                "What is the difference between compliance and obedience?",
                "Both involve explicit requests, but obedience is a response to a direct order from a legitimate authority figure, whereas compliance is a response to a request from an equal with no power differential.",
            ),
            (
                "What is the difference between normative and informational social influence?",
                "Normative influence is conforming to gain acceptance or avoid rejection, without genuine belief change. Informational influence is conforming because you believe the group has superior knowledge, producing real belief change.",
            ),
            (
                "What is actor-observer bias?",
                "The tendency to attribute our own behavior to situational factors while attributing others' behavior to dispositional factors, applying different standards to ourselves versus others.",
            ),
            (
                "What is self-serving bias?",
                "The tendency to attribute one's own successes to internal dispositional factors and one's failures to external situational factors, which protects self-esteem.",
            ),
            (
                "What is the just-world hypothesis?",
                "The belief that the world is fair and people get what they deserve, which can lead to blaming victims for their misfortunes because outcomes are assumed to reflect character.",
            ),
            (
                "What is cognitive dissonance?",
                "The mental discomfort of holding contradictory beliefs or acting in conflict with one's beliefs. The discomfort motivates a change in attitude or behavior to restore consistency.",
            ),
            (
                "What is the difference between prejudice and discrimination?",
                "Prejudice is an internal negative attitude toward a group, while discrimination is the external, observable behavior of treating people unfavorably based on group membership.",
            ),
            (
                "What is the difference between social facilitation and social loafing?",
                "Social facilitation is improved performance on simple or well-learned tasks when others are present. Social loafing is reduced individual effort in a group when personal contribution is not identifiable.",
            ),
            (
                "What is the bystander effect and what mechanism explains it?",
                "The bystander effect is the reduced likelihood that any individual will help a victim when others are present. It is driven by diffusion of responsibility, where accountability is spread across the group.",
            ),
            (
                "What is the difference between an in-group and an out-group?",
                "An in-group is a social group one identifies with (us), associated with favoritism. An out-group is one an individual does not belong to (them), associated with the out-group homogeneity effect and stereotyping.",
            ),
            (
                "What is stereotype threat?",
                "The phenomenon in which awareness of a negative stereotype about one's group creates anxiety that impairs performance, causing individuals to inadvertently confirm the stereotype.",
            ),
            (
                "What is a self-fulfilling prophecy?",
                "When an expectation about a person or situation influences behavior in ways that cause the expectation to come true, such as a teacher's low expectations leading a student to underperform.",
            ),
            (
                "What is the difference between achieved and ascribed status?",
                "Achieved status is a social position earned through effort or choice, such as occupation. Ascribed status is assigned at birth or involuntarily, such as race or sex, over which one has no control.",
            ),
            (
                "What is the difference between role conflict and role strain?",
                "Role conflict is tension between the demands of two or more different roles a person holds. Role strain is tension from incompatible demands within a single role.",
            ),
            (
                "What is the core claim of conflict theory (Marx and Weber)?",
                "Conflict theory holds that society is structured around competition for limited resources, producing inequality between groups. Marx emphasized economic class, while Weber added prestige and power.",
            ),
            (
                "What is symbolic interactionism?",
                "A micro-level sociological perspective that examines how people create and interpret shared meanings through symbols and everyday social interaction, shaping their sense of reality.",
            ),
            (
                "What does the demographic transition model describe?",
                "A model describing how populations shift from high birth and death rates to low birth and death rates as a society industrializes and develops, changing population growth over stages.",
            ),
            (
                "What is social stratification and how does it relate to socioeconomic status?",
                "Social stratification is the hierarchical ranking of people into layers based on unequal access to resources. Socioeconomic status reflects an individual's position based on income, education, and occupation.",
            ),
            (
                "What is the difference between cultural capital and social capital?",
                "Cultural capital is the non-financial assets such as education, skills, and cultural knowledge that promote mobility. Social capital is the advantages gained through social networks and relationships.",
            ),
            (
                "What is the difference between the sympathetic and parasympathetic nervous systems?",
                "The sympathetic system drives fight-or-flight (raising heart rate, using norepinephrine at effectors). The parasympathetic system drives rest-and-digest (lowering heart rate, using acetylcholine).",
            ),
            (
                "What is the difference between a receptor agonist and an antagonist?",
                "An agonist binds a receptor and activates it, mimicking the natural ligand. An antagonist binds a receptor and blocks it without activating it, preventing the natural ligand from acting.",
            ),
            (
                "What is the difference between internal and external validity?",
                "Internal validity is the degree to which changes in the dependent variable can be causally attributed to the independent variable. External validity is the degree to which findings generalize beyond the study.",
            ),
            (
                "What is the difference between reliability and validity?",
                "Reliability is the consistency of a measurement, while validity is its accuracy. A measure can be reliable without being valid, but it cannot be valid without being reliable.",
            ),
            (
                "What is the difference between longitudinal and cross-sectional studies?",
                "Longitudinal studies follow the same individuals over time, while cross-sectional studies compare different groups at a single point in time. Cross-sectional designs are vulnerable to cohort effects.",
            ),
            (
                "What is the difference between the placebo and nocebo effects?",
                "The placebo effect is a real improvement caused by the expectation of receiving treatment, while the nocebo effect is a worsening of symptoms caused by negative expectations about an inert treatment.",
            ),
            (
                "What is the difference between the biomedical and biopsychosocial models of health?",
                "The biomedical model explains illness purely through biological factors, while the biopsychosocial model integrates biological, psychological, and social factors to understand health and disease.",
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
            (
                "What does the CARS section fundamentally test?",
                "It is an argument-analysis test, not a reading-comprehension or recall test. It measures your ability to comprehend, reason, and apply using only what the passage provides.",
            ),
            (
                "What is the CARS section format (passages, questions, time)?",
                "Nine passages, 53 questions, and a 90-minute time limit, with no breaks within the section.",
            ),
            (
                "What is the target time budget per CARS passage?",
                "About 10 minutes total: roughly 3 to 4 minutes reading and building your map, and 6 to 7 minutes answering the questions.",
            ),
            (
                "What are the three CARS question categories and their approximate weights?",
                "Foundations of Comprehension (~30%), Reasoning Within the Text (~30%), and Reasoning Beyond the Text (~40%).",
            ),
            (
                "Which CARS question category carries the most weight, and why does it matter?",
                "Reasoning Beyond the Text at about 40 percent. It is the highest-weight category and the one students most consistently miss, so strategy gains here matter most.",
            ),
            (
                "What subject areas do CARS passages cover?",
                "About 50 percent humanities (philosophy, literature, art, ethics, history, religion) and 50 percent social sciences (psychology, sociology, economics, political science, anthropology).",
            ),
            (
                "What question types fall under Foundations of Comprehension?",
                "Main idea and primary purpose, specific detail or direct retrieval, vocabulary in context, and tone or author attitude.",
            ),
            (
                "What question types fall under Reasoning Within the Text?",
                "Strengthen or weaken, function of a paragraph, logical structure, and identifying an assumption.",
            ),
            (
                "What question types fall under Reasoning Beyond the Text?",
                "Application or analogy, author response to new scenarios, new-information strengthen or weaken, and prediction.",
            ),
            (
                "What are the four elements of a CARS passage map?",
                "The central thesis in one sentence, the author's tone and stance, each paragraph's purpose, and location flags for key details.",
            ),
            (
                "What is the central thesis in passage mapping?",
                "A one-sentence statement of the author's main argument that serves as your anchor for filtering every question. It captures the specific claim, not just the topic.",
            ),
            (
                "Why is identifying tone so important in CARS?",
                "Misreading tone is a common source of wrong answers on main idea and author-attitude questions. Tone reflects the author's relationship to the argument, not just positive or negative.",
            ),
            (
                "What are the main tone or stance categories to watch for?",
                "Defending a position, critiquing a position, presenting multiple views, uncertain or exploratory, and sarcastic or ironic.",
            ),
            (
                "What are location flags in a passage map?",
                "Paragraph-level mental tags for named studies, statistics, dates, or defined terms that might appear in questions, so you can go straight there instead of re-reading.",
            ),
            (
                "How does the AAMC define a valid CARS inference?",
                "A conclusion that is necessarily true given the passage and would not be true without it. Valid inferences are modest, tightly bounded, and traceable to specific passage language.",
            ),
            (
                "What is the deletion test for inference questions?",
                "Ask whether the answer would still seem plausible if you deleted the passage. If yes, it relies on outside knowledge and should be eliminated.",
            ),
            (
                "Why is applying outside knowledge almost always wrong in CARS?",
                "An answer can be factually true in the real world yet unsupported by the passage. If support cannot be found directly in the text, the answer is almost certainly wrong.",
            ),
            (
                "What is the 'too broad' wrong-answer trap?",
                "An answer that accurately describes the topic but overstates the scope, such as generalizing a claim about one composer to all Enlightenment composers.",
            ),
            (
                "What is the 'too narrow' wrong-answer trap?",
                "An answer that accurately describes one paragraph but misses the overall argument. It often uses language taken directly from the passage, which makes it feel correct.",
            ),
            (
                "What is the 'outside the text' wrong-answer trap?",
                "An answer that is factually true in the real world but not supported by or derivable from the passage. It most often catches students with strong background knowledge.",
            ),
            (
                "What is the 'opposite' or 'contradicts author' trap?",
                "An answer that describes the position the author is arguing against. It is common in passages where the author presents a counterargument before rebutting it.",
            ),
            (
                "How does extreme or absolute language signal a wrong answer?",
                "Words like always, never, all, none, only, or completely usually overstate a claim the passage supports only in a more qualified form.",
            ),
            (
                "What is the 'misrepresents tone' trap?",
                "An answer that gets the topic right but describes the author's attitude incorrectly, such as calling a cautiously optimistic author an enthusiastic advocate.",
            ),
            (
                "What is the 'half-right, half-wrong' trap?",
                "An answer whose first clause accurately describes the passage but whose second clause contradicts it or adds unsupported information. Read every word, including the ending.",
            ),
            (
                "What is the 'distractor detail' trap?",
                "An answer that uses real words or names from the passage but applies them incorrectly. The detail is genuine, but its use is wrong.",
            ),
            (
                "What one-word habit makes elimination systematic?",
                "Justify each eliminated choice with a single word such as 'too narrow', 'extreme', or 'outside'. This keeps reasoning explicit and prevents second-guessing.",
            ),
            (
                "What is the 90-second rule in CARS?",
                "Never spend more than 90 seconds on a single question. If you have not eliminated down to one answer, mark your best choice, flag it, and move on.",
            ),
            (
                "How should you modulate reading speed within a passage?",
                "Slow down at thesis statements, the first sentence of paragraphs, and explicit opinion language; speed up through supporting examples.",
            ),
            (
                "How should you decide passage order in the CARS section?",
                "Spend about 30 seconds scanning first lines and start with passages you find easiest to engage. You may do passages in any order and flag to return.",
            ),
            (
                "In what four predictable ways does the AAMC build wrong main idea answers?",
                "Too narrow (describes one paragraph), too broad (overstates scope), contradicts the tone, or distorts the claim. A precise one-sentence thesis eliminates all four.",
            ),
            (
                "What is the difference between top-down and bottom-up passages?",
                "Top-down passages state the main claim in the first paragraph or two, then support it; bottom-up passages build evidence first and synthesize the thesis near the end.",
            ),
            (
                "Where do most main idea mistakes happen, and how do you avoid them?",
                "In bottom-up passages, where students anchor to the opening problem setup and never revise. After finishing, ask whether the author's position became clearer and update your thesis.",
            ),
            (
                "Which transition words signal a contrast or counterargument?",
                "However, yet, but, in contrast, on the other hand, despite, nevertheless, although, and while.",
            ),
            (
                "Which transition words signal added evidence or support?",
                "Moreover, furthermore, additionally, in fact, indeed, as evidence, for instance, and specifically.",
            ),
            (
                "Which phrases signal a concede-then-rebut structure?",
                "Admittedly, granted, it is true that... but, and even though... still. They mark the author acknowledging a point before pushing back.",
            ),
            (
                "Which transition words indicate causation or a conclusion drawn?",
                "Therefore, thus, consequently, as a result, this suggests, and it follows that.",
            ),
            (
                "In CARS, is the correct answer usually the clearly correct one?",
                "No. The correct answer is frequently the least wrong option, not the most satisfying or sophisticated one. The three wrong options are almost always definitively wrong.",
            ),
            (
                "What mindset should you use when evaluating answer choices?",
                "Build a case for elimination, not selection. Ask whether anything in the passage makes each choice definitively wrong; the option with no case against it is your answer.",
            ),
            (
                "What is the defining feature of the CARS blind review protocol?",
                "You do not look at your scores before completing a second pass. You re-read the passage and reconstruct your reasoning for every choice before checking solutions.",
            ),
            (
                "What two steps make CARS review 'deep' that most students skip?",
                "Re-reading the passage before reviewing any questions, and reconstructing your reasoning for every answer choice before looking at the solution.",
            ),
            (
                "Why does blind review work better than ordinary review?",
                "Redoing a passage without knowing your scores removes the defensive reasoning that leads students to justify why they were 'almost right' instead of analyzing their actual error.",
            ),
            (
                "What is the 'no-return' rule in CARS reading?",
                "CARS passages are meant to be read once, carefully. Going back to re-read is a warning sign that the first read did not produce genuine comprehension and it costs time.",
            ),
            (
                "Why is CARS described as a 'listening exam' rather than a knowledge exam?",
                "Its goal is to set aside your own opinions and fully understand exactly what the author is saying on their terms, acting as a blank slate rather than an evaluator.",
            ),
            (
                "What clinical skill does CARS mirror?",
                "A patient interaction: the author is the patient and their argument is the history, so your job is to hear them accurately without projection, assumptions, or personal commentary.",
            ),
            (
                "Why must a CARS strategy be applied consistently on every passage?",
                "Variation is the enemy of skill consolidation. Changing your approach on hard passages means the strategy never gets tested under real conditions where it matters most.",
            ),
            (
                "How should you handle a CARS passage on an unfamiliar topic?",
                "You do not need topic knowledge; all needed information is in the passage. Apply your mapping method mechanically, since argument structure does not depend on subject matter.",
            ),
            (
                "What question keywords do students most often misread, causing errors even with a good map?",
                "Negation and qualifier words such as NOT, EXCEPT, most likely, and 'would the author disagree'. Missing them leads you to answer a different question than the one asked.",
            ),
            (
                "What is a paragraph purpose note and how long should it be?",
                "A one-to-three-word tag of a paragraph's structural role, such as 'introduces thesis' or 'counterargument'. It should take only a few seconds per paragraph.",
            ),
        ],
    ),
];

/// A named topic paired with the fronts of the seeded cards that belong to it.
pub(crate) type Topic = (&'static str, &'static [&'static str]);

/// A section deck name paired with the topics it is broken into for
/// exam-coverage reporting.
pub(crate) type SectionTopics = (&'static str, &'static [Topic]);

/// Topic taxonomy used by exam-coverage reporting. Each of the four scored
/// MCAT sections is broken into finer-grained topics, and every topic lists the
/// fronts of the seeded cards that teach it. A topic counts as "covered" once
/// every one of its cards present in the collection has been mastered.
///
/// This is kept in sync with [`MCAT_SECTIONS`] by the `topics_match_seed_cards`
/// test: every front here must correspond to exactly one seeded card, and every
/// seeded card must be assigned to exactly one topic.
pub(crate) const MCAT_TOPICS: &[SectionTopics] = &[
    (
        "MCAT::Biology & Biochemistry",
        &[
            (
                "Amino Acids & Protein Structure",
                &[
                    "What are the four levels of protein structure?",
                    "What type of bond links amino acids together in a protein?",
                    "What four groups are attached to the central alpha carbon of an amino acid?",
                    "What is the isoelectric point (pI) of an amino acid?",
                    "Which amino acid is the only achiral one, and what configuration do the chiral ones have?",
                    "What stabilizes the secondary structure of a protein?",
                    "What bond forms cystine, and how does it stabilize tertiary structure?",
                    "Why does burying hydrophobic R groups in a protein's core favor folding?",
                ],
            ),
            (
                "Enzymes & Kinetics",
                &[
                    "What is the difference between competitive and noncompetitive enzyme inhibition?",
                    "Do enzymes change the delta G or equilibrium of a reaction?",
                    "What does the Michaelis constant Km represent?",
                    "How does competitive inhibition affect Vmax and Km?",
                    "How does noncompetitive inhibition affect Vmax and Km?",
                    "How does uncompetitive inhibition affect Vmax and Km?",
                    "What is a zymogen?",
                    "What does phosphorylation do to catabolic versus anabolic enzymes?",
                ],
            ),
            (
                "DNA Replication & Repair",
                &[
                    "Which enzyme unwinds the DNA double helix during replication?",
                    "What is the role of the enzyme telomerase?",
                    "During which phase of the cell cycle is DNA replicated?",
                    "State Chargaff's rules for DNA base composition.",
                    "What does it mean that DNA replication is semiconservative?",
                    "How does nucleotide excision repair fix thymine dimers?",
                    "What are telomeres and how are they maintained?",
                ],
            ),
            (
                "Gene Expression & Molecular Genetics",
                &[
                    "What is the central dogma of molecular biology?",
                    "What are the start and stop codons of the genetic code?",
                    "What does it mean that the genetic code is degenerate?",
                    "What happens to introns and exons during mRNA processing?",
                    "In translation, what happens at the A, P, and E sites of the ribosome?",
                    "How does an inducible operon like the lac operon work?",
                    "How does a repressible operon like the trp operon work?",
                    "Contrast oncogenes and tumor suppressor genes.",
                ],
            ),
            (
                "Classical Genetics & Evolution",
                &[
                    "State Mendel's law of segregation.",
                    "What is the difference between codominance and incomplete dominance?",
                    "Give the Hardy-Weinberg equations and what p and q represent.",
                    "What experiment confirmed that DNA, not protein, is the genetic material?",
                ],
            ),
            (
                "Glycolysis & Cellular Respiration",
                &[
                    "What is the net yield of glycolysis per glucose?",
                    "What is the rate-limiting enzyme of glycolysis and how is it regulated?",
                    "Where does the citric acid cycle occur and what does it produce per acetyl-CoA?",
                    "What is the rate-limiting enzyme of the citric acid cycle?",
                    "How does ATP synthase generate ATP in oxidative phosphorylation?",
                    "What is the final electron acceptor of the electron transport chain?",
                ],
            ),
            (
                "Metabolic Pathways & Bioenergetics",
                &[
                    "Which enzymes bypass the irreversible steps of glycolysis in gluconeogenesis?",
                    "What glycosidic bonds do glycogen synthase and the branching enzyme create?",
                    "What is the main product and rate-limiting enzyme of the pentose phosphate pathway?",
                    "Describe the steps of beta-oxidation of fatty acids.",
                    "When and where are ketone bodies produced?",
                    "Which amino acids are solely ketogenic?",
                    "Why is ATP hydrolysis energetically favorable?",
                ],
            ),
            (
                "Cell Structure & Transport",
                &[
                    "What organelles define eukaryotic cells and where does the ETC occur in each cell type?",
                    "Compare the rough and smooth endoplasmic reticulum.",
                    "Distinguish gap junctions, tight junctions, and desmosomes.",
                    "How does the Na+/K+ pump move ions and what type of transport is it?",
                ],
            ),
            (
                "Microbiology & Viruses",
                &[
                    "Distinguish gram-positive and gram-negative bacteria.",
                    "What is a retrovirus and what enzyme does it require?",
                    "Contrast the lytic and lysogenic bacteriophage life cycles.",
                ],
            ),
            (
                "Physiology & Development",
                &[
                    "Which hormone lowers blood glucose, and where is it produced?",
                    "What are the three primary germ layers and one derivative of each?",
                    "Which neurotransmitters do the two divisions of the autonomic nervous system use?",
                ],
            ),
        ],
    ),
    (
        "MCAT::Chemistry & Physics",
        &[
            (
                "Acids, Bases & Buffers",
                &[
                    "What is the difference between a strong and weak acid?",
                    "How is pH defined in terms of hydrogen ion concentration?",
                    "What is the relationship between pH and pOH at 25 C?",
                    "What does the Henderson-Hasselbalch equation state for a buffer?",
                    "What makes a solution a good buffer and where is it most effective?",
                    "Name the common strong acids that fully dissociate in water.",
                    "What relationship links Ka and Kb of a conjugate acid-base pair?",
                    "What defines the equivalence point of an acid-base titration?",
                ],
            ),
            (
                "Chemical Kinetics & Equilibrium",
                &[
                    "What does a catalyst do to a chemical reaction?",
                    "What does Le Chatelier's principle predict when a system at equilibrium is stressed?",
                    "How is the equilibrium constant Keq written for aA + bB -> cC + dD?",
                    "What is the reaction quotient Q used for?",
                    "What does the order of a reaction tell you about rate dependence?",
                ],
            ),
            (
                "Thermodynamics",
                &[
                    "What distinguishes an exothermic from an endothermic reaction?",
                    "What determines whether a reaction is spontaneous according to Gibbs free energy?",
                    "When is a reaction spontaneous at all temperatures based on enthalpy and entropy?",
                    "What does Hess's law allow you to do?",
                ],
            ),
            (
                "Gases",
                &[
                    "What is the ideal gas law?",
                    "What are the conditions defined as STP and the molar volume of an ideal gas there?",
                    "What does Dalton's law of partial pressures state?",
                ],
            ),
            (
                "Periodic Trends & Atomic Structure",
                &[
                    "How do electronegativity and atomic radius trend across the periodic table?",
                    "How does first ionization energy trend on the periodic table?",
                    "What do the four quantum numbers n, l, ml, and ms describe?",
                ],
            ),
            (
                "Electrochemistry & Redox",
                &[
                    "In a galvanic (voltaic) cell, what happens at the anode and cathode?",
                    "How is cell potential related to spontaneity and free energy?",
                    "What is the sign convention for the electrodes in an electrolytic cell?",
                    "State the basic rules for assigning oxidation numbers.",
                ],
            ),
            (
                "Solutions & Solubility",
                &[
                    "What are colligative properties and give examples?",
                    "What does the solubility product Ksp represent?",
                    "What is the common ion effect?",
                ],
            ),
            (
                "Organic Chemistry",
                &[
                    "How do you identify a carbonyl group and name two functional groups containing it?",
                    "What are enantiomers and how do they differ physically?",
                    "What characterizes an SN2 reaction mechanism?",
                    "What characterizes an SN1 reaction mechanism?",
                    "What is the difference between a nucleophile and an electrophile?",
                    "How does resonance affect molecular stability?",
                    "What is the difference between constitutional isomers and stereoisomers?",
                    "What products form when primary versus secondary alcohols are oxidized?",
                    "Which IR absorptions are diagnostic for O-H and C=O groups?",
                ],
            ),
            (
                "Mechanics",
                &[
                    "State Newton's second law of motion.",
                    "What is the SI unit of force and how is it defined?",
                    "Give the kinematic equation relating displacement, initial velocity, acceleration, and time.",
                    "How is work defined for a constant force?",
                    "What is the work-energy theorem?",
                    "What does conservation of mechanical energy state?",
                    "How are impulse and momentum related?",
                    "What determines the period of a simple pendulum and a mass on a spring?",
                ],
            ),
            (
                "Electricity & Magnetism",
                &[
                    "State Coulomb's law for the force between two point charges.",
                    "How is electric field related to force on a charge?",
                    "State Ohm's law and the formula for electrical power.",
                    "How do resistors combine in series versus parallel?",
                    "How is the capacitance of a capacitor defined?",
                ],
            ),
            (
                "Waves, Sound & Optics",
                &[
                    "What is the relationship between wavelength and frequency of light?",
                    "What is the relationship among wave speed, frequency, and wavelength?",
                    "What is the Doppler effect for sound?",
                    "How does sound intensity level in decibels relate to intensity?",
                    "State the thin lens and mirror equation and the sign of focal length.",
                    "State Snell's law of refraction.",
                    "What is total internal reflection and when does it occur?",
                ],
            ),
            (
                "Fluids",
                &[
                    "How is pressure in a static fluid related to depth?",
                    "State Archimedes' principle of buoyancy.",
                    "What does the continuity equation state for an incompressible fluid?",
                    "What does Bernoulli's principle relate in a flowing fluid?",
                ],
            ),
        ],
    ),
    (
        "MCAT::Psychology & Sociology",
        &[
            (
                "Learning & Conditioning",
                &[
                    "What is classical conditioning?",
                    "What is operant conditioning?",
                    "What is the difference between positive and negative reinforcement?",
                    "What is the difference between positive and negative punishment?",
                    "How do stimulus generalization and discrimination differ in conditioning?",
                    "What is observational learning and who is it associated with?",
                ],
            ),
            (
                "Sensation & Perception",
                &[
                    "What is the difference between sensation and perception?",
                    "What is an absolute threshold?",
                    "What is the difference threshold (JND) and what law governs it?",
                    "How does top-down processing differ from bottom-up processing?",
                ],
            ),
            (
                "Memory",
                &[
                    "What is the difference between explicit and implicit memory?",
                    "How do semantic and episodic memory differ?",
                    "How does short-term (working) memory differ from long-term memory?",
                    "What is the difference between proactive and retroactive interference?",
                    "What is the difference between encoding and retrieval?",
                ],
            ),
            (
                "Cognitive Development",
                &[
                    "Distinguish between assimilation and accommodation (Piaget).",
                    "What distinguishes Piaget's concrete operational stage from the formal operational stage?",
                ],
            ),
            (
                "Motivation & Emotion",
                &[
                    "Name Maslow's hierarchy of needs from bottom to top.",
                    "What does the Yerkes-Dodson (arousal) law state about motivation?",
                    "What is the difference between intrinsic and extrinsic motivation?",
                    "How does the Schachter-Singer two-factor theory of emotion work?",
                    "How do the James-Lange and Cannon-Bard theories of emotion differ?",
                ],
            ),
            (
                "Stress & Coping",
                &[
                    "What are the three stages of the general adaptation syndrome?",
                    "What is the difference between problem-focused and emotion-focused coping?",
                ],
            ),
            (
                "Sleep & Consciousness",
                &["What are the stages of sleep characterized by EEG patterns?"],
            ),
            (
                "Psychological Disorders",
                &[
                    "What is the difference between major depressive disorder and bipolar disorder?",
                    "What is the diathesis-stress model of psychological disorders?",
                ],
            ),
            (
                "Attribution & Social Cognition",
                &[
                    "Define the fundamental attribution error.",
                    "What is actor-observer bias?",
                    "What is self-serving bias?",
                    "What is the just-world hypothesis?",
                    "What is cognitive dissonance?",
                    "What is a self-fulfilling prophecy?",
                ],
            ),
            (
                "Social Influence",
                &[
                    "Define social facilitation.",
                    "What is the difference between conformity and compliance?",
                    "What is the difference between compliance and obedience?",
                    "What is the difference between normative and informational social influence?",
                    "What is the difference between social facilitation and social loafing?",
                    "What is the bystander effect and what mechanism explains it?",
                ],
            ),
            (
                "Attitudes & Prejudice",
                &[
                    "What is the difference between prejudice and discrimination?",
                    "What is the difference between an in-group and an out-group?",
                    "What is stereotype threat?",
                ],
            ),
            (
                "Self & Identity",
                &[
                    "What is the looking-glass self?",
                    "What is impression management (dramaturgy)?",
                ],
            ),
            (
                "Social Structure & Stratification",
                &[
                    "What is the difference between achieved and ascribed status?",
                    "What is the difference between role conflict and role strain?",
                    "What is the core claim of conflict theory (Marx and Weber)?",
                    "What is symbolic interactionism?",
                    "What does the demographic transition model describe?",
                    "What is social stratification and how does it relate to socioeconomic status?",
                    "What is the difference between cultural capital and social capital?",
                ],
            ),
            (
                "Socialization & Groups",
                &[
                    "What is socialization?",
                    "What is the difference between a primary and secondary group?",
                ],
            ),
            (
                "Research Methods",
                &[
                    "What is the difference between an independent and dependent variable?",
                    "What is the difference between internal and external validity?",
                    "What is the difference between reliability and validity?",
                    "What is the difference between longitudinal and cross-sectional studies?",
                    "What is the difference between the placebo and nocebo effects?",
                ],
            ),
            (
                "Biological Basis of Behavior",
                &[
                    "What is the difference between the sympathetic and parasympathetic nervous systems?",
                    "What is the difference between a receptor agonist and an antagonist?",
                ],
            ),
            (
                "Health & Behavior Models",
                &["What is the difference between the biomedical and biopsychosocial models of health?"],
            ),
        ],
    ),
    (
        "MCAT::Critical Analysis & Reasoning (CARS)",
        &[
            (
                "CARS Overview & Format",
                &[
                    "What does the CARS section test?",
                    "What does the CARS section fundamentally test?",
                    "What is the CARS section format (passages, questions, time)?",
                    "What is the target time budget per CARS passage?",
                    "What subject areas do CARS passages cover?",
                ],
            ),
            (
                "Question Categories & Types",
                &[
                    "What are the three CARS question categories and their approximate weights?",
                    "Which CARS question category carries the most weight, and why does it matter?",
                    "What question types fall under Foundations of Comprehension?",
                    "What question types fall under Reasoning Within the Text?",
                    "What question types fall under Reasoning Beyond the Text?",
                ],
            ),
            (
                "Passage Mapping",
                &[
                    "Why is it important to read for structure, not just content, in CARS?",
                    "What are the four elements of a CARS passage map?",
                    "What is the central thesis in passage mapping?",
                    "What are location flags in a passage map?",
                    "What is a paragraph purpose note and how long should it be?",
                ],
            ),
            (
                "Tone & Stance",
                &[
                    "What is the author's tone?",
                    "Why is identifying tone so important in CARS?",
                    "What are the main tone or stance categories to watch for?",
                ],
            ),
            (
                "Inference & Reasoning",
                &[
                    "What is the difference between an inference and a stated detail?",
                    "What is the main idea (thesis) of a passage?",
                    "How does the AAMC define a valid CARS inference?",
                    "What is the deletion test for inference questions?",
                    "Why is applying outside knowledge almost always wrong in CARS?",
                ],
            ),
            (
                "Wrong-Answer Traps",
                &[
                    "What is the 'too broad' wrong-answer trap?",
                    "What is the 'too narrow' wrong-answer trap?",
                    "What is the 'outside the text' wrong-answer trap?",
                    "What is the 'opposite' or 'contradicts author' trap?",
                    "How does extreme or absolute language signal a wrong answer?",
                    "What is the 'misrepresents tone' trap?",
                    "What is the 'half-right, half-wrong' trap?",
                    "What is the 'distractor detail' trap?",
                ],
            ),
            (
                "Elimination Strategy",
                &[
                    "What strategy helps with 'strengthen' or 'weaken' questions?",
                    "How should you approach answer choices with extreme wording (e.g., 'always', 'never')?",
                    "What is the purpose of an 'except' or 'least' question?",
                    "What one-word habit makes elimination systematic?",
                    "In CARS, is the correct answer usually the clearly correct one?",
                    "What mindset should you use when evaluating answer choices?",
                ],
            ),
            (
                "Timing & Pacing",
                &[
                    "What is the 90-second rule in CARS?",
                    "How should you modulate reading speed within a passage?",
                    "How should you decide passage order in the CARS section?",
                ],
            ),
            (
                "Main Idea Strategy",
                &[
                    "In what four predictable ways does the AAMC build wrong main idea answers?",
                    "What is the difference between top-down and bottom-up passages?",
                    "Where do most main idea mistakes happen, and how do you avoid them?",
                ],
            ),
            (
                "Transition Words",
                &[
                    "Which transition words signal a contrast or counterargument?",
                    "Which transition words signal added evidence or support?",
                    "Which phrases signal a concede-then-rebut structure?",
                    "Which transition words indicate causation or a conclusion drawn?",
                ],
            ),
            (
                "Review & Mindset",
                &[
                    "What is the defining feature of the CARS blind review protocol?",
                    "What two steps make CARS review 'deep' that most students skip?",
                    "Why does blind review work better than ordinary review?",
                    "What is the 'no-return' rule in CARS reading?",
                    "Why is CARS described as a 'listening exam' rather than a knowledge exam?",
                    "What clinical skill does CARS mirror?",
                    "Why must a CARS strategy be applied consistently on every passage?",
                    "How should you handle a CARS passage on an unfamiliar topic?",
                    "What question keywords do students most often misread, causing errors even with a good map?",
                ],
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::collections::HashSet;

    use super::*;

    /// The topic taxonomy must stay in lock-step with the seeded cards:
    /// every seeded card is assigned to exactly one topic, and every front
    /// listed in the taxonomy corresponds to a real seeded card.
    #[test]
    fn topics_match_seed_cards() {
        // Build a lookup of section deck name -> set of seeded card fronts.
        let mut seeded: HashMap<&str, HashSet<&str>> = HashMap::new();
        for (deck_name, cards) in MCAT_SECTIONS {
            let set = seeded.entry(*deck_name).or_default();
            for (front, _back) in *cards {
                assert!(
                    set.insert(*front),
                    "duplicate seeded front in {deck_name}: {front}"
                );
            }
        }

        // Every topic front must map to a seeded card, and each card must be
        // claimed by exactly one topic.
        let mut claimed: HashMap<&str, HashSet<&str>> = HashMap::new();
        for (deck_name, topics) in MCAT_TOPICS {
            let seeded_fronts = seeded
                .get(deck_name)
                .unwrap_or_else(|| panic!("MCAT_TOPICS references unknown section {deck_name}"));
            let claimed_set = claimed.entry(*deck_name).or_default();
            for (topic_name, fronts) in *topics {
                for front in *fronts {
                    assert!(
                        seeded_fronts.contains(front),
                        "topic '{topic_name}' in {deck_name} lists a front with no seeded card: {front}"
                    );
                    assert!(
                        claimed_set.insert(*front),
                        "front assigned to more than one topic in {deck_name}: {front}"
                    );
                }
            }
        }

        // Every seeded card must be assigned to some topic.
        for (deck_name, fronts) in &seeded {
            let claimed_set = claimed
                .get(*deck_name)
                .unwrap_or_else(|| panic!("section {deck_name} has no topics"));
            for front in fronts {
                assert!(
                    claimed_set.contains(front),
                    "seeded card in {deck_name} is not assigned to any topic: {front}"
                );
            }
        }
    }
}
