import { useState } from 'react'
import type { Route, ScopeRequirement, TeamRequirement } from '../types'

interface RouteFormProps {
  initialValues?: Route
  onSubmit: (values: Route) => void
  isLoading: boolean
}

export function RouteForm({
  initialValues,
  onSubmit,
  isLoading,
}: RouteFormProps) {
  const [host, setHost] = useState(initialValues?.host || '')
  const [path, setPath] = useState(initialValues?.path || '')
  const [roles, setRoles] = useState<string[]>(
    initialValues?.require.roles || []
  )
  const [permissions, setPermissions] = useState<string[]>(
    initialValues?.require.permissions || []
  )
  const [scopes, setScopes] = useState<ScopeRequirement[]>(
    initialValues?.require.scopes || []
  )
  const [teams, setTeams] = useState<TeamRequirement[]>(
    initialValues?.require.teams || []
  )

  // New role input
  const [newRole, setNewRole] = useState('')

  // New permission input
  const [newPermission, setNewPermission] = useState('')

  // New scope inputs
  const [newResourceType, setNewResourceType] = useState('')
  const [newAction, setNewAction] = useState('')
  const [newResourceId, setNewResourceId] = useState('')

  // New team inputs
  const [newTeamId, setNewTeamId] = useState('')
  const [newTeamName, setNewTeamName] = useState('')

  // Validation state
  const [errors, setErrors] = useState<Record<string, string>>({})

  // Handle form submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    // Validate form
    const newErrors: Record<string, string> = {}

    if (!host.trim()) {
      newErrors.host = 'Host is required'
    }

    if (!path.trim()) {
      newErrors.path = 'Path is required'
    } else if (!path.startsWith('/')) {
      newErrors.path = 'Path must start with /'
    }

    // Check if at least one requirement is specified
    if (
      roles.length === 0 &&
      permissions.length === 0 &&
      scopes.length === 0 &&
      teams.length === 0
    ) {
      newErrors.require =
        'At least one requirement (roles, permissions, scopes, or teams) must be specified'
    }

    // If there are errors, show them and don't submit
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors)
      return
    }

    // Clear errors
    setErrors({})

    // Create the route object
    const route: Route = {
      id: initialValues?.id,
      host,
      path,
      require: {
        roles: roles.length > 0 ? roles : undefined,
        permissions: permissions.length > 0 ? permissions : undefined,
        scopes: scopes.length > 0 ? scopes : undefined,
        teams: teams.length > 0 ? teams : undefined,
      },
    }

    // Submit the form
    onSubmit(route)
  }

  // Add a new role
  const addRole = () => {
    if (newRole.trim() && !roles.includes(newRole.trim())) {
      setRoles([...roles, newRole.trim()])
      setNewRole('')
    }
  }

  // Remove a role
  const removeRole = (role: string) => {
    setRoles(roles.filter((r) => r !== role))
  }

  // Add a new permission
  const addPermission = () => {
    if (newPermission.trim() && !permissions.includes(newPermission.trim())) {
      setPermissions([...permissions, newPermission.trim()])
      setNewPermission('')
    }
  }

  // Remove a permission
  const removePermission = (permission: string) => {
    setPermissions(permissions.filter((p) => p !== permission))
  }

  // Add a new scope
  const addScope = () => {
    if (newResourceType.trim() && newAction.trim()) {
      const newScope: ScopeRequirement = {
        resource_type: newResourceType.trim(),
        action: newAction.trim(),
      }

      if (newResourceId.trim()) {
        newScope.resource_id = newResourceId.trim()
      }

      setScopes([...scopes, newScope])
      setNewResourceType('')
      setNewAction('')
      setNewResourceId('')
    }
  }

  // Remove a scope
  const removeScope = (index: number) => {
    setScopes(scopes.filter((_, i) => i !== index))
  }

  // Add a new team
  const addTeam = () => {
    if (newTeamId.trim() || newTeamName.trim()) {
      const newTeam: TeamRequirement = {}

      if (newTeamId.trim()) {
        newTeam.id = newTeamId.trim()
      }

      if (newTeamName.trim()) {
        newTeam.name = newTeamName.trim()
      }

      setTeams([...teams, newTeam])
      setNewTeamId('')
      setNewTeamName('')
    }
  }

  // Remove a team
  const removeTeam = (index: number) => {
    setTeams(teams.filter((_, i) => i !== index))
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Host */}
      <div>
        <label htmlFor="host" className="label">
          Host
        </label>
        <input
          id="host"
          type="text"
          className={`input ${errors.host ? 'border-red-500' : ''}`}
          value={host}
          onChange={(e) => setHost(e.target.value)}
          placeholder="e.g., api.example.com or *.example.com"
          disabled={isLoading}
        />
        {errors.host && (
          <p className="text-red-500 text-sm mt-1">{errors.host}</p>
        )}
      </div>

      {/* Path */}
      <div>
        <label htmlFor="path" className="label">
          Path
        </label>
        <input
          id="path"
          type="text"
          className={`input ${errors.path ? 'border-red-500' : ''}`}
          value={path}
          onChange={(e) => setPath(e.target.value)}
          placeholder="e.g., /api/users or /admin/*"
          disabled={isLoading}
        />
        {errors.path && (
          <p className="text-red-500 text-sm mt-1">{errors.path}</p>
        )}
      </div>

      {/* Requirements error */}
      {errors.require && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p>{errors.require}</p>
        </div>
      )}

      {/* Roles */}
      <div>
        <h3 className="text-lg font-medium mb-2">Roles</h3>
        <div className="flex mb-2">
          <input
            type="text"
            className="input mr-2"
            value={newRole}
            onChange={(e) => setNewRole(e.target.value)}
            placeholder="Enter a role"
            disabled={isLoading}
          />
          <button
            type="button"
            className="btn btn-secondary"
            onClick={addRole}
            disabled={isLoading || !newRole.trim()}
          >
            Add
          </button>
        </div>

        {roles.length > 0 ? (
          <div className="flex flex-wrap gap-2 mt-2">
            {roles.map((role) => (
              <div
                key={role}
                className="bg-blue-100 text-blue-800 px-3 py-1 rounded-full flex items-center"
              >
                <span>{role}</span>
                <button
                  type="button"
                  className="ml-2 text-blue-600 hover:text-blue-800"
                  onClick={() => removeRole(role)}
                  disabled={isLoading}
                >
                  &times;
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 italic">No roles specified</p>
        )}
      </div>

      {/* Permissions */}
      <div>
        <h3 className="text-lg font-medium mb-2">Permissions</h3>
        <div className="flex mb-2">
          <input
            type="text"
            className="input mr-2"
            value={newPermission}
            onChange={(e) => setNewPermission(e.target.value)}
            placeholder="Enter a permission"
            disabled={isLoading}
          />
          <button
            type="button"
            className="btn btn-secondary"
            onClick={addPermission}
            disabled={isLoading || !newPermission.trim()}
          >
            Add
          </button>
        </div>

        {permissions.length > 0 ? (
          <div className="flex flex-wrap gap-2 mt-2">
            {permissions.map((permission) => (
              <div
                key={permission}
                className="bg-green-100 text-green-800 px-3 py-1 rounded-full flex items-center"
              >
                <span>{permission}</span>
                <button
                  type="button"
                  className="ml-2 text-green-600 hover:text-green-800"
                  onClick={() => removePermission(permission)}
                  disabled={isLoading}
                >
                  &times;
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 italic">No permissions specified</p>
        )}
      </div>

      {/* Scopes */}
      <div>
        <h3 className="text-lg font-medium mb-2">Scopes</h3>
        <div className="grid grid-cols-3 gap-2 mb-2">
          <input
            type="text"
            className="input"
            value={newResourceType}
            onChange={(e) => setNewResourceType(e.target.value)}
            placeholder="Resource Type"
            disabled={isLoading}
          />
          <input
            type="text"
            className="input"
            value={newAction}
            onChange={(e) => setNewAction(e.target.value)}
            placeholder="Action"
            disabled={isLoading}
          />
          <input
            type="text"
            className="input"
            value={newResourceId}
            onChange={(e) => setNewResourceId(e.target.value)}
            placeholder="Resource ID (optional)"
            disabled={isLoading}
          />
        </div>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={addScope}
          disabled={isLoading || !newResourceType.trim() || !newAction.trim()}
        >
          Add Scope
        </button>

        {scopes.length > 0 ? (
          <div className="mt-2 space-y-2">
            {scopes.map((scope, index) => (
              <div
                key={index}
                className="bg-purple-100 text-purple-800 px-3 py-2 rounded-md flex justify-between items-center"
              >
                <span>
                  {scope.resource_type}:{scope.action}
                  {scope.resource_id && `:${scope.resource_id}`}
                </span>
                <button
                  type="button"
                  className="text-purple-600 hover:text-purple-800"
                  onClick={() => removeScope(index)}
                  disabled={isLoading}
                >
                  &times;
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 italic mt-2">No scopes specified</p>
        )}
      </div>

      {/* Teams */}
      <div>
        <h3 className="text-lg font-medium mb-2">Teams</h3>
        <div className="grid grid-cols-2 gap-2 mb-2">
          <input
            type="text"
            className="input"
            value={newTeamId}
            onChange={(e) => setNewTeamId(e.target.value)}
            placeholder="Team ID"
            disabled={isLoading}
          />
          <input
            type="text"
            className="input"
            value={newTeamName}
            onChange={(e) => setNewTeamName(e.target.value)}
            placeholder="Team Name"
            disabled={isLoading}
          />
        </div>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={addTeam}
          disabled={isLoading || (!newTeamId.trim() && !newTeamName.trim())}
        >
          Add Team
        </button>

        {teams.length > 0 ? (
          <div className="mt-2 space-y-2">
            {teams.map((team, index) => (
              <div
                key={index}
                className="bg-yellow-100 text-yellow-800 px-3 py-2 rounded-md flex justify-between items-center"
              >
                <span>
                  {team.id && `ID: ${team.id}`}
                  {team.id && team.name && ' | '}
                  {team.name && `Name: ${team.name}`}
                </span>
                <button
                  type="button"
                  className="text-yellow-600 hover:text-yellow-800"
                  onClick={() => removeTeam(index)}
                  disabled={isLoading}
                >
                  &times;
                </button>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 italic mt-2">No teams specified</p>
        )}
      </div>

      {/* Submit button */}
      <div className="pt-4">
        <button
          type="submit"
          className="btn btn-primary w-full"
          disabled={isLoading}
        >
          {isLoading
            ? 'Saving...'
            : initialValues
              ? 'Update Route'
              : 'Create Route'}
        </button>
      </div>
    </form>
  )
}
