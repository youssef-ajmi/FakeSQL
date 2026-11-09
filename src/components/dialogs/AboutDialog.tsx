import { Database, X } from "lucide-react";

interface AboutDialogProps {
  onClose: () => void;
}

export function AboutDialog({ onClose }: AboutDialogProps) {
  return (
    <div className="modal-overlay" onMouseDown={onClose}>
      <div className="dialog about-dialog" onMouseDown={(event) => event.stopPropagation()}>
        <div className="dialog-header">
          <span className="dialog-title">
            <Database size={15} /> À propos
          </span>
          <button type="button" className="icon-button small" onClick={onClose} title="Fermer">
            <X size={14} />
          </button>
        </div>
        <div className="dialog-body about-body">
          <div className="about-logo">FQ</div>
          <h2>FakeSQL Studio</h2>
          <p className="about-version">Version 0.1.0</p>
          <p>
            Un petit IDE inspiré de JetBrains pour écrire et exécuter des requêtes FakeSQL sur
            des fichiers texte.
          </p>
          <div className="shortcut-grid">
            <span>Exécuter la requête</span>
            <kbd>Ctrl+Enter</kbd>
            <span>Ouvrir un dossier</span>
            <kbd>Ctrl+O</kbd>
            <span>Nouvelle requête</span>
            <kbd>Ctrl+N</kbd>
            <span>Fermer l'onglet</span>
            <kbd>Ctrl+W</kbd>
          </div>
        </div>
        <div className="dialog-footer">
          <button type="button" className="primary-button" onClick={onClose}>
            Fermer
          </button>
        </div>
      </div>
    </div>
  );
}
