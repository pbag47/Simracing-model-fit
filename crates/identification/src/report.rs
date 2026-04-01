use crate::filter::FilterStats;
use crate::evaluator::EvaluationResult;
use models::vehicle::bicycle::BicycleParams;

#[derive(Debug)]
pub struct IdentificationReport {
    pub params:     BicycleParams,
    pub filter:     FilterStats,
    pub evaluation: EvaluationResult,
}

impl IdentificationReport {
    pub fn print(&self) {
        // println!("\n{'='*50}");
        self.filter.print_summary();
        println!();
        self.evaluation.print_summary();
        println!();
        println!("=== Paramètres bicyclette ===");
        println!("  Cf (avant)     : {:.0} N/rad", self.params.cornering_stiffness_front);
        println!("  Cr (arrière)   : {:.0} N/rad", self.params.cornering_stiffness_rear);
        println!("  Iz             : {:.0} kg·m²",  self.params.yaw_inertia);
        println!("  Masse          : {:.0} kg",      self.params.mass);
        println!("  L_f            : {:.3} m",       self.params.l_front);
        println!("  L_r            : {:.3} m",       self.params.l_rear);
        println!("  K (sous-vir.)  : {:.4} rad·s²/m²",
            self.params.understeer_gradient());
        if let Some(v_crit) = self.params.critical_speed() {
            println!("  v_critique     : {:.1} km/h ⚠️  (survireur)",
                v_crit * 3.6);
        } else {
            println!("  Comportement   : sous-vireur (stable)");
        }
    }
}