<details>
<summary>XSD contract: <code>MemberValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="MemberValueType">
		<xs:annotation>
			<xs:documentation>Allows for a ditinct reference or a wildcard expression for selecting codes from a codelist.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:extension base="WildcardedMemberValueType">
				<xs:attribute name="cascadeValues" type="common:CascadeSelectionType" use="optional">
					<xs:annotation>
						<xs:documentation>Indicates whether child codes should be selected when the codelist is hierarchical. Possible values are true (include the selected and child codes), false (only include the selected code(s)), and excluderoot (include the children but not the selected code(s)).</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
