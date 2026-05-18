use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io::{self, Write};
use crate::Args;

pub fn run_completion() {
    println!("\nHermes Shell Completion Generator");
    println!("═════════════════════════════════════\n");
    println!("Select your shell to generate the auto-completion script:");
    println!("  1. Bash");
    println!("  2. Zsh");
    println!("  3. Fish");
    println!("  4. PowerShell");
    println!("  5. Elvish");
    println!();

    print!("  Choice [1-5]: ");
    io::stdout().flush().ok();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).ok();
    let choice = choice.trim().parse::<usize>().unwrap_or(1);

    let shell = match choice {
        1 => Shell::Bash,
        2 => Shell::Zsh,
        3 => Shell::Fish,
        4 => Shell::PowerShell,
        5 => Shell::Elvish,
        _ => Shell::Bash,
    };

    println!("\n# --- COPY THE SCRIPT BELOW ---");
    let mut cmd = Args::command();
    generate(shell, &mut cmd, "hermes", &mut io::stdout());
    println!("# --- END OF SCRIPT ---");
    
    println!("\nTip: To load this automatically on startup, redirect this output to a file and source it:");
    match shell {
        Shell::Bash => println!("  hermes completion > ~/.hermes/completion.bash\n  echo \"source ~/.hermes/completion.bash\" >> ~/.bashrc"),
        Shell::Zsh => println!("  hermes completion > ~/.hermes/completion.zsh\n  echo \"source ~/.hermes/completion.zsh\" >> ~/.zshrc"),
        Shell::Fish => println!("  hermes completion > ~/.config/fish/completions/hermes.fish"),
        _ => {}
    }
}
