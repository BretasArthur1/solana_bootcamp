'use client'

import { PublicKey } from '@solana/web3.js'
import { useCrudappProgram, useCrudappProgramAccount } from './crudapp-data-access'
import { useState, useEffect } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'

export function CrudappCreate() {
  const [title, setTitle] = useState('');
  const [message, setMessage] = useState('');
  const [error, setError] = useState<string | null>(null);
  const { createEntry } = useCrudappProgram();
  const { publicKey } = useWallet();

  const isFormValid = title.trim() !== '' && message.trim() !== '';

  const handleSubmit = async () => {
    if (!publicKey) {
      setError('Please connect your wallet');
      return;
    }
    if (isFormValid) {
      try {
        await createEntry.mutateAsync({ title, message, owner: publicKey });
        setTitle('');
        setMessage('');
        setError(null);
      } catch (err) {
        setError('Failed to create entry. Please try again.');
      }
    }
  };

  return (
    <div className="p-4">
      <h1 className="text-2xl font-bold mb-4">Create Journal Entry</h1>
      {error && <div className="alert alert-error mb-4">{error}</div>}
      <input
        type="text"
        placeholder="Title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        className="input input-bordered w-full max-w-xs mb-2"
      />
      <textarea
        placeholder="Message"
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        className="textarea textarea-bordered w-full max-w-xs mb-2"
      ></textarea>
      <button
        onClick={handleSubmit}
        disabled={createEntry.isPending || !isFormValid}
        className="btn btn-primary"
      >
        {createEntry.isPending ? 'Creating...' : 'Create'}
      </button>
    </div>
  );
}

export function CrudappList() {
  const { accounts, getProgramAccount } = useCrudappProgram()

  if (getProgramAccount.isLoading) {
    return <span className="loading loading-spinner loading-lg"></span>
  }
  if (!getProgramAccount.data?.value) {
    return (
      <div className="alert alert-info flex justify-center">
        <span>Program account not found. Make sure you have deployed the program and are on the correct cluster.</span>
      </div>
    )
  }
  return (
    <div className={'space-y-6'}>
      {accounts.isLoading ? (
        <span className="loading loading-spinner loading-lg"></span>
      ) : accounts.data?.length ? (
        <div className="grid md:grid-cols-2 gap-4">
          {accounts.data?.map((account) => (
            <CrudappCard key={account.publicKey.toString()} account={account.publicKey} />
          ))}
        </div>
      ) : (
        <div className="text-center">
          <h2 className={'text-2xl'}>No accounts</h2>
          No accounts found. Create one above to get started.
        </div>
      )}
    </div>
  )
}

function CrudappCard({ account }: { account: PublicKey }) {
  const { accountQuery, updateEntry, deleteEntry } = useCrudappProgramAccount({ account });
  const { publicKey } = useWallet();
  const [message, setMessage] = useState('');
  const [error, setError] = useState<string | null>(null);
  const title = accountQuery.data?.title;
  const isFormValid = message.trim() !== '';

  const handleSubmit = async () => {
    if (!publicKey) {
      setError('Please connect your wallet');
      return;
    }
    if (isFormValid && title) {
      try {
        await updateEntry.mutateAsync({ title, message, owner: publicKey });
        setMessage('');
        setError(null);
      } catch (err) {
        setError('Failed to update entry. Please try again.');
      }
    }
  };

  return accountQuery.isLoading ? (
    <div className="flex justify-center items-center h-full">
      <span className="loading loading-spinner loading-lg"></span>
    </div>
  ) : (
    <div className="card bg-base-100 shadow-xl p-4">
      <h2 className="card-title text-xl font-bold mb-2">{title}</h2>
      {error && <div className="alert alert-error mb-4">{error}</div>}
      <p className="mb-4">{accountQuery.data?.message}</p>
      <div className="flex space-x-2">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          className="input input-bordered w-full"
          placeholder="Update your message"
        />
        <button
          onClick={handleSubmit}
          disabled={updateEntry.isPending || !isFormValid}
          className="btn btn-primary"
        >
          {updateEntry.isPending ? 'Updating...' : 'Update'}
        </button>
      </div>
    </div>
  );
}
