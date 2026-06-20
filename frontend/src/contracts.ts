// Declarative description of each deployed contract and its functions.
// The UI renders forms straight from this config, so "merging" all three
// contracts is just three entries in CONTRACTS.

import { DEFAULT_PESO_BRIDGE_ID } from './stellar';

export type ArgType = 'address' | 'token' | 'i128' | 'u32' | 'u64' | 'vec_i128';

export type FnDef = {
  name: string;
  kind: 'read' | 'write';
  label: string;
  desc?: string;
  args: { name: string; type: ArgType; placeholder?: string }[];
};

export type ContractDef = {
  key: string;
  name: string;
  emoji: string;
  tagline: string;
  /** Deployed testnet contract id. Empty = user pastes it in the UI. */
  defaultId: string;
  /** Short ordered hint of the happy-path flow. */
  flow: string[];
  fns: FnDef[];
};

export const CONTRACTS: ContractDef[] = [
  {
    key: 'peso_bridge',
    name: 'PesoBridge',
    emoji: '🌉',
    tagline: 'Cross-border B2B invoice settlement',
    defaultId: DEFAULT_PESO_BRIDGE_ID,
    flow: ['initialize(token)', 'issue_invoice', 'pay_invoice', 'get_invoice'],
    fns: [
      { name: 'initialize', kind: 'write', label: 'Initialize', desc: 'Set the settlement token (once).',
        args: [{ name: 'token', type: 'token', placeholder: 'C… token contract id' }] },
      { name: 'issue_invoice', kind: 'write', label: 'Issue invoice', desc: 'Exporter creates a pending invoice; returns its id.',
        args: [
          { name: 'exporter', type: 'address' },
          { name: 'buyer', type: 'address' },
          { name: 'amount', type: 'i128', placeholder: '10000000 (= 1 XLM)' },
        ] },
      { name: 'pay_invoice', kind: 'write', label: 'Pay invoice', desc: 'Buyer pays; settles to exporter. Freighter must be the buyer.',
        args: [{ name: 'id', type: 'u64' }] },
      { name: 'cancel_invoice', kind: 'write', label: 'Cancel invoice', desc: 'Exporter cancels a still-pending invoice.',
        args: [{ name: 'id', type: 'u64' }] },
      { name: 'get_invoice', kind: 'read', label: 'Get invoice', args: [{ name: 'id', type: 'u64' }] },
      { name: 'invoice_status', kind: 'read', label: 'Invoice status', args: [{ name: 'id', type: 'u64' }] },
      { name: 'total_settled', kind: 'read', label: 'Total settled', args: [] },
    ],
  },
  {
    key: 'suweldo_chain',
    name: 'SuweldoChain',
    emoji: '⚡',
    tagline: 'Instant SME payroll with on-chain payslips',
    defaultId: '', // paste your deployed suweldo_chain id
    flow: ['initialize(admin, token)', 'add_employee', 'fund_payroll', 'run_payroll', 'get_payslip'],
    fns: [
      { name: 'initialize', kind: 'write', label: 'Initialize', desc: 'Set the employer admin + payout token (once).',
        args: [
          { name: 'admin', type: 'address' },
          { name: 'token', type: 'token', placeholder: 'C… token contract id' },
        ] },
      { name: 'add_employee', kind: 'write', label: 'Add employee',
        args: [
          { name: 'employee', type: 'address' },
          { name: 'salary', type: 'i128', placeholder: 'per-period pay, base units' },
        ] },
      { name: 'remove_employee', kind: 'write', label: 'Remove employee',
        args: [{ name: 'employee', type: 'address' }] },
      { name: 'fund_payroll', kind: 'write', label: 'Fund payroll', desc: 'Move USDC into the payroll escrow.',
        args: [
          { name: 'from', type: 'address' },
          { name: 'amount', type: 'i128' },
        ] },
      { name: 'run_payroll', kind: 'write', label: 'Run payroll', desc: 'Pay every active worker for this period in one batch.',
        args: [{ name: 'period', type: 'u32' }] },
      { name: 'get_employee', kind: 'read', label: 'Get employee', args: [{ name: 'employee', type: 'address' }] },
      { name: 'get_payslip', kind: 'read', label: 'Get payslip',
        args: [{ name: 'employee', type: 'address' }, { name: 'period', type: 'u32' }] },
      { name: 'is_period_paid', kind: 'read', label: 'Is period paid', args: [{ name: 'period', type: 'u32' }] },
      { name: 'total_paid', kind: 'read', label: 'Total paid', args: [] },
    ],
  },
  {
    key: 'takdang_bayad',
    name: 'Takdang Bayad',
    emoji: '🤝',
    tagline: 'Freelancer milestone escrow',
    defaultId: '', // paste your deployed takdang_bayad id
    flow: ['initialize(token)', 'create_job', 'fund_milestone', 'approve_milestone', 'get_milestone'],
    fns: [
      { name: 'initialize', kind: 'write', label: 'Initialize', desc: 'Set the escrow token (once).',
        args: [{ name: 'token', type: 'token', placeholder: 'C… token contract id' }] },
      { name: 'create_job', kind: 'write', label: 'Create job', desc: 'Client opens a job with a list of milestone amounts; returns job id.',
        args: [
          { name: 'client', type: 'address' },
          { name: 'freelancer', type: 'address' },
          { name: 'amounts', type: 'vec_i128', placeholder: 'comma-separated, e.g. 100,200' },
        ] },
      { name: 'fund_milestone', kind: 'write', label: 'Fund milestone', desc: 'Client locks a milestone into escrow.',
        args: [{ name: 'job_id', type: 'u64' }, { name: 'milestone', type: 'u32' }] },
      { name: 'approve_milestone', kind: 'write', label: 'Approve milestone', desc: 'Release a funded milestone to the freelancer.',
        args: [{ name: 'job_id', type: 'u64' }, { name: 'milestone', type: 'u32' }] },
      { name: 'refund_milestone', kind: 'write', label: 'Refund milestone', desc: 'Reclaim a funded-but-unapproved milestone.',
        args: [{ name: 'job_id', type: 'u64' }, { name: 'milestone', type: 'u32' }] },
      { name: 'get_job', kind: 'read', label: 'Get job', args: [{ name: 'job_id', type: 'u64' }] },
      { name: 'get_milestone', kind: 'read', label: 'Get milestone',
        args: [{ name: 'job_id', type: 'u64' }, { name: 'milestone', type: 'u32' }] },
      { name: 'is_released', kind: 'read', label: 'Is released',
        args: [{ name: 'job_id', type: 'u64' }, { name: 'milestone', type: 'u32' }] },
    ],
  },
];
