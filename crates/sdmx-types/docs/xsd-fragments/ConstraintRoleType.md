<details>
<summary>XSD contract: <code>ConstraintRoleType</code> (SDMX 3.0)</summary>

```xml
	<xs:simpleType name="ConstraintRoleType">
		<xs:annotation>
			<xs:documentation>ConstraintRoleType defines a list of roles for a content constraint. A constraint can state which data is present or which content is allowed for the constraint attachment.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="xs:string">
			<xs:enumeration value="Allowed">
				<xs:annotation>
					<xs:documentation>The constraint contains the allowed values for attachable object.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Actual">
				<xs:annotation>
					<xs:documentation>The constraints contains the actual data present for the attachable object.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
		</xs:restriction>
	</xs:simpleType>
```

</details>
